---
type: ADR
id: ADR-085
title: "ec_distann: Single Global Vamana Graph with Hash-Placed Records and Coordinator Hop-Round Search"
status: PROPOSED
impact: Establishes the fifth access method (ec_distann) and the successor distributed-search architecture to partitioned SPIRE routing. Governs FR-075..FR-083, NFR-014, NFR-016..NFR-020, StR-008. Relates to ADR-049/054/056/067/068 (reused machinery) and records the measured rejection of the partitioned-routing lane.
date: 2026-07-06
---
# ADR-085: ec_distann — Single Global Vamana Graph with Hash-Placed Records and Coordinator Hop-Round Search

## Context

The SPIRE remediation program measured, on a release-verified substrate, that
partitioned routing cannot reach the distributed gate. With every lever
applied — epoch caching (40–44% latency-floor cut), leaf-only ranking, SPANN
closure assignment, distance-ratio pruning, rerank economy — holding 0.99
distinct recall still required scanning **35.7% of corpus row-instances at
50k and 78.7% at 100k** (target ≤5%), with the fraction growing with corpus
size. Evidence (task numbers 141–146 collide across lanes on `main`; cite by
branch + packet path):

- `reviews/task-144/012-release-matrix-decision/` on branch
  `task-144-spire-closure-ratio-pruning` (closure/ratio gate failure).
- `reviews/task-145/012-phase3-do-not-promote-decision/` on branch
  `task-145-spire-rerank-economy-low-probe` (all economy levers
  inert/negative).
- `reviews/task-146/006-anchor-results/` on branch
  `task-146-spire-honest-pareto-confirmation`: release anchors IVF 100k
  0.9980 @ 37.6 ms, HNSW 100k 0.9795 @ 20.4 ms.

The root cause is architectural: a lossy partition-level routing decision
must be hedged wider as recall targets rise and corpora grow. DistributedANN
(**arXiv:2509.06046**, the durable citation; convenience local copy
`~/dev_bak/papers/distributedann-2509.06046.pdf`; same research group as
SPANN/SPIRE, replaced partition-routing in Bing production) inverts the
design: distribute the storage, keep the index whole. One global Vamana
graph; records hash-placed; query cost = beam × hops, corpus-independent.

Milestones M0–M5 referenced throughout this batch are defined normatively in
`plan/design/distann-global-graph-architecture.md` (M0 single-node parity,
M1 stitch, M2 two-node read path, M3 lifecycle+faults, M4 bench gate, M5
incremental insert).

## Decision

Build `ec_distann` as a fifth access method:

1. **One global Vamana graph** over all indexed vectors. No partitions, no
   routing decision, no boundary replication; the entire
   duplicate/distinct-recall problem class of the partitioned lane does not
   exist here (one logical record per vector, global vec_id).
2. **Lean node records + co-placed heap rerank** (FR-076/D11): each record
   holds a coarse search code + adjacency + embedded compressed neighbor
   codes, so one read expands a node and scores all neighbors; the
   full-precision vector is NOT in the record but in a co-placed heap row
   read node-locally for exact rerank. The single-node degenerate path may use
   the base-table row; a physical multinode epoch uses FR-078's immutable
   AM-owned epoch row tier. This is the `ec_diskann` coarse-in-index /
   exact-from-heap split, sharded without live cross-node base-table TIDs.
3. **Hash placement** (FR-078): `hash(vec_id) mod roster`; placement affects
   load balance only.
4. **Sharded build + stitch** (FR-077): closure-overlap clustering (the
   Task 144 distance-ratio machinery repurposed from query-time crutch to
   build-time scaffold) → parallel per-shard Vamana builds → union + re-prune
   stitch.
5. **Coordinator head index** (FR-080) + **H batched hop rounds** (FR-081)
   over the lifted SPIRE CustomScan/transport/epoch machinery
   (ADR-067/068-adjacent code, post-142 pooling).
6. **Committed write path** (FR-083): tombstone deletes now; full incremental
   distributed insert as the program's final milestone (operator decision
   2026-07-06 — in scope, not conditional).

## Sub-Decisions

- **D1 — Neighbor-code duplication over on-demand fetch.** Embedding R
  neighbor codes per record trades disk for one-read expansion. Corrected
  arithmetic for the measured D7 default is based on the implementation's
  RaBitQ **1-bit** stride, not a hypothetical 4-bit stride. At dim=1536 the
  stride is `ceil(1536/8) + 12 = 204 B`; with R=32, FR-076's exact record
  formula is `20 + 204 + 32×8 + 32×204 = 7,008 B`, or about **1.14×** the
  6,144-byte raw f32 vector before page/tuple/directory overhead. D11 keeps the
  full-precision vector out of that numerator and stores it once in the
  co-placed epoch row tier. The NFR-018 4.0× threshold therefore has measured
  implementation headroom for the default, while TurboQuant's 4-bit
  dimension-wide stride remains a distinct high-space/fallback case. Actual
  graph, TOAST, directory, and metadata bytes at 10k/50k/100k still decide the
  gate; arithmetic is not evidence.
- **D2 — Gate substrate: loopback multi-instance**, matching how the
  IVF/HNSW anchors were measured; one informational injected-latency
  (netem) run accompanies the gate for external validity. H×RTT sensitivity
  is reported, not gated. Because NFR-017 is the program kill criterion and
  transport data first exists at M2, M0/M1 SHALL include a **kill-check
  spike**: single-node recall-vs-H curve × the measured per-round transport
  cost of the existing SPIRE pipeline, projecting multinode p50 before M2 is
  built.
- **D3 — Head-index size C: fixed cap reloption** (`head_index_cap`,
  default 4096, breadth-first sample unioned across shard top layers);
  recall sensitivity measured at M0 before the default is frozen. **Measured
  outcome (2026-07-12, `reviews/task-179/038-head-cap-sensitivity/`): retain
  4096.** Physical recall for C=64/256/4096 was 0.995/0.995/1.000 at 10k,
  0.975/0.980/0.980 at 50k, and 0.920/0.945/0.950 at 100k; all topology and
  remote-engagement gates passed. **Task 182 amendment (2026-07-16):** retain C
  = 4096 but permit an explicit generation policy
  `training_landmarks_exact`. It selects the persisted cap from 200 ordered,
  digest-bound, disjoint training queries and exact-scores the bounded persisted
  landmarks. The original BFS/Vamana policy remains the default and legacy
  interpretation. The trained policy is generation metadata, never a query GUC
  or filesystem dependency, and remains default-off pending production A/B.
- **D4 — BatANN baton passing rejected for now** (operator decision:
  orchestrator-pull only). Reopen trigger: M2 measurement showing hop-round
  RTT ≥ 50% of multinode p50 at gate-relevant BW/H.
- **D5 — Interim insert posture: bounded exact-scan delta buffer** (visible
  same-statement, drained at next epoch build), chosen over erroring so DML
  tests exercise visibility semantics early; the end state is FR-083's
  incremental insert with next-epoch neighbor-edge repair semantics. The
  interim posture is not an acceptable terminal state: the program closes
  only with incremental insert landed or an explicit operator descope.
- **D6 — vec_id = hash64(source_identity) with build-time collision
  detection** (fail the build on collision; probability negligible at
  research scales), over dense per-epoch assignment — keeps vec_ids stable
  across epochs and nodes without a per-record directory.
- **D7 — Neighbor-code codec: GroupedPq default** (closest to the paper's
  OPQ; already a `QuantCodec`), exposed as the `neighbor_code_format`
  reloption with rabitq and turboquant as measured alternatives at M0.
  **M0 measured outcome (2026-07-07, `reviews/task-162/002-m0-bench-cells/`):
  default pinned to `rabitq`.** GroupedPq tops out at 0.9245 recall@10 at
  50k where rabitq reaches 0.9950 at comparable latency, and turboquant's
  768 B codes exceed page capacity at the default R=32 (the D1 fallback
  scenario). The reloption keeps all three formats selectable.
- **D8 — Stitch memory: stream by vec_id group.** Shard outputs are sorted
  by vec_id; the stitch merges shard streams and never holds more than one
  vec_id group plus prune working set in memory.
- **D9 — Termination: fixed H with convergence early-exit** (FR-081); BW×H
  stays the hard cap so NFR-019 is assertable per query.
- **D10 — Published-epoch mutation model** (added after failure-domain
  review): within a Published epoch, records and adjacency are immutable
  except monotonic tombstone-flag sets, delta-buffer appends (D5), and
  incremental-insert appends with back-edge amendments (FR-083). The
  fingerprint attests to roster/placement/format/build-time record set, not
  the mutable delta state; physical reclaim + edge repair happen only at the
  next epoch build; in-flight scans may observe pre- or post-amendment
  adjacency (both valid), with per-record write atomicity. Normative text in
  FR-082.
- **D11 — Co-placed heap rerank (A) over an index-resident shipped rerank
  tier (B).** The rerank fidelity source is the co-placed full-precision
  row, read node-locally via `heap_tid`; in a physical multinode epoch this is
  the immutable AM-owned FR-078 row tier, while the single-node degenerate path
  may use its base heap. The index record carries no vector
  (FR-076/FR-078/FR-079). **Why (A), not SPIRE's (B):** SPIRE ships a
  compressed rerank code into index storage because its wide-leaf scan
  scores *many* candidates per leaf, so a per-candidate heap fetch is a
  random-read storm — index-resident codes are forced by scan-fraction.
  distann has no scan-fraction: NFR-019 caps per-query work at BW×H expanded
  records independent of corpus size. Exact-vector row reads are bounded by
  live expansions (tombstones may skip). For an unqualified LIMIT with no
  tombstone or snapshot-visibility skips, final payload reads follow the fixed
  window bound `min(D, W × ceil(k/W))`; `t` skipped ranked slots change it to
  `min(D, W × ceil((k+t)/W))`. Qual rejection can require additional proven
  candidates under D12's fixed corpus-independent deepening ceiling. Task 191
  reconciles NFR-019's older unconditional `+ k` wording before D12 becomes the
  production default. An implementation may reuse a row read but correctness
  does not assume it. The thing that forced (B)
  is exactly the failure mode distann is designed not to have. **What (A)
  buys:** the full vector is stored once (in the heap tier), never
  duplicated into the index → −1.0× raw off per-record amplification (D1);
  the index is byte-identical in shape to `ec_diskann`, so M0 single-node is
  literally the `ec_diskann` path (index = codes, heap = vectors, exact
  rerank from the local heap) and inherits its parity almost for free;
  fidelity is exact, not code-approximated. **Honest cost:** in a multi-node
  deployment expansion is two node-local reads (record + heap row) rather
  than one inline read, and the vector must be co-placed with the record
  (FR-078) — an extra placement obligation. Both reads stay local (no extra
  network round-trip), and the read-count delta is bounded by BW×H, so it is
  dominated by transport. The rerank source is kept conceptually pluggable
  (a future `index`-tier mode could serve a same-node deployment that wants
  to avoid the heap detoast), but `ec_diskann`/`ec_distann` default to — and
  in practice only use — the local heap source (base heap for the single-node
  degenerate path, frozen epoch heap for multinode).
- **D12 — Executor-driven, fixed-window final payload materialization.** Task
  184 selected deterministic global-ranked windows of 10 over eager
  materialization of the entire final candidate set. A scan retains each
  candidate's `vec_id`; when the executor reaches the first not-yet-materialized
  remote candidate in a window, the coordinator concurrently fetches only the
  pending remote payloads in that proven ranked prefix. Qual rejection deepens
  to the next window, still bounded by the already-ranked candidate set. Local
  and remote candidates keep one global order. Projection attnums, snapshot and
  generation fencing, row identity, and owner failure behavior are unchanged;
  a failure during a later batch fails the query and cannot turn a prefix into
  a complete result. Each request is capped by the current proven prefix and
  total qual-driven materialization remains capped by the existing fixed
  deepening ceiling derived once from the initial search bar
  (`max(initial × 64, 1024)`), independent of corpus size. The fixed window
  size is 10—adaptive sizing, 20/40 alternatives, prefetch, and pipelining are
  not part of this decision. Task 184's matched
  10k/50k/100k evidence preserved recall at 0.9990/0.9685/0.9625 and reduced
  warm mean latency from 34.10/36.00/38.30 ms to 20.70/22.20/22.40 ms, with
  better p50/p95/p99/max at every scale and unchanged storage/build. Evidence:
  `reviews/task-184/003-isolated-candidate/` and
  `reviews/task-184/004-full-scale-decision/`. Task 191 owns the normative FR
  update and production-default implementation. Until Task 191 lands, normal
  builds remain eager and Task 184's implementation remains benchmark-only.

## Consequences

- Query-time work becomes corpus-independent (NFR-019) — the property the
  partitioned lane could not deliver; the gate (NFR-017) is now a
  head-to-head against single-instance IVF on the same protocol.
- Build becomes the hard distributed problem (stitch correctness is the
  least-proven step — FR-077 carries property-test obligations), and disk
  pays the D1 amplification — now the R× neighbor-code block alone, since
  D11 keeps the 1.0×-raw vector out of the record.
- Placement gains a co-location obligation (D11/FR-078): each vec_id's
  full-precision AM-owned epoch row must land on the same node as its record;
  multi-node expansion is two node-local reads (record + heap) instead of
  one inline read. Both stay local, so latency is still H × per-round
  transport, not read-bound.
- Latency floor is H × per-round transport cost; the post-142 pooling work
  is a prerequisite, and D4's reopen trigger guards the risk.
- Final-payload remote work is driven by executor demand in bounded ranked
  windows (D12), avoiding payload transport for candidates that never survive
  `LIMIT`/qual consumption. This changes scan execution semantics but not the
  persisted format, placement, wire endpoint, or failure contract.
- The SPIRE partitioned lane remains shelved-with-evidence; its
  CustomScan/epoch/placement/transport machinery is reused, not discarded.

## Rejected Alternatives

- **Continue iterating partitioned SPIRE** (ADR-051/060 escalations):
  rejected — the Task 144 scan-fraction scaling (2.96% → 35.7% → 78.7%)
  shows the hedging cost is architectural, not a tuning residue.
- **BatANN-style baton passing** (arXiv:2512.09331): deferred per D4.
- **IVF-only distributed**: viable fallback recorded in the Task 146 lane;
  does not address the scan-fraction scaling at higher recall targets.
- **Learned routing over partitions** (ADR-052/053 lineage): out of scope
  for this program — it iterates the rejected partition-routing
  architecture rather than removing the routing decision.
