---
type: ADR
id: ADR-085
title: "ec_distann: Single Global Vamana Graph with Hash-Placed Records and Coordinator Hop-Round Search"
status: PROPOSED
impact: Establishes the fifth access method (ec_distann) and the successor distributed-search architecture to partitioned SPIRE routing. Governs FR-075..FR-083, NFR-017..NFR-020, StR-008. Relates to ADR-049/054/056/067/068 (reused machinery) and records the measured rejection of the partitioned-routing lane.
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
(arXiv:2509.06046; local copy
`~/dev_bak/papers/distributedann-2509.06046.pdf`; same research group as
SPANN/SPIRE, replaced partition-routing in Bing production) inverts the
design: distribute the storage, keep the index whole. One global Vamana
graph; records hash-placed; query cost = beam × hops, corpus-independent.

## Decision

Build `ec_distann` as a fifth access method:

1. **One global Vamana graph** over all indexed vectors. No partitions, no
   routing decision, no boundary replication; the entire
   duplicate/distinct-recall problem class of the partitioned lane does not
   exist here (one logical record per vector, global vec_id).
2. **Self-sufficient node records** (FR-076): full vector + adjacency +
   embedded compressed neighbor codes, so one read expands a node and scores
   all neighbors.
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
  neighbor codes per record trades disk for one-read expansion. Arithmetic at
  dim=1536, R=32, rabitq-class 4-bit codes (~768 B/code): ≈24.6 KB/record vs
  6.1 KB raw vector — dominated by the code block, bounded by NFR-018's ≤4×
  budget only with compact codes; the reference paper's ~10× used
  full-precision-adjacent OPQ at higher degree. Validate by M0 storage
  measurement; fallback (adjacency-only records, codes piggybacked on
  expansion responses) is a format-version change, acceptable under the
  research rebuild posture.
- **D2 — Gate substrate: loopback multi-instance**, matching how the
  IVF/HNSW anchors were measured; one informational injected-latency
  (netem) run accompanies the gate for external validity. H×RTT sensitivity
  is reported, not gated.
- **D3 — Head-index size C: fixed cap reloption** (`head_index_cap`,
  default 4096, breadth-first sample unioned across shard top layers);
  recall sensitivity measured at M0 before the default is frozen.
- **D4 — BatANN baton passing rejected for now** (operator decision:
  orchestrator-pull only). Reopen trigger: M2 measurement showing hop-round
  RTT ≥ 50% of multinode p50 at gate-relevant BW/H.
- **D5 — Interim insert posture: bounded exact-scan delta buffer** (visible
  same-statement, drained at next epoch build), chosen over erroring so DML
  tests exercise visibility semantics early; the end state is FR-083's
  incremental insert with next-epoch neighbor-edge repair semantics.
- **D6 — vec_id = hash64(source_identity) with build-time collision
  detection** (fail the build on collision; probability negligible at
  research scales), over dense per-epoch assignment — keeps vec_ids stable
  across epochs and nodes without a per-record directory.
- **D7 — Neighbor-code codec: GroupedPq default** (closest to the paper's
  OPQ; already a `QuantCodec`), exposed as the `neighbor_code_format`
  reloption with rabitq and turboquant as measured alternatives at M0.
- **D8 — Stitch memory: stream by vec_id group.** Shard outputs are sorted
  by vec_id; the stitch merges shard streams and never holds more than one
  vec_id group plus prune working set in memory.
- **D9 — Termination: fixed H with convergence early-exit** (FR-081); BW×H
  stays the hard cap so NFR-019 is assertable per query.

## Consequences

- Query-time work becomes corpus-independent (NFR-019) — the property the
  partitioned lane could not deliver; the gate (NFR-017) is now a
  head-to-head against single-instance IVF on the same protocol.
- Build becomes the hard distributed problem (stitch correctness is the
  least-proven step — FR-077 carries property-test obligations), and disk
  pays the D1 amplification.
- Latency floor is H × per-round transport cost; the post-142 pooling work
  is a prerequisite, and D4's reopen trigger guards the risk.
- The SPIRE partitioned lane remains shelved-with-evidence; its
  CustomScan/epoch/placement/transport machinery is reused, not discarded.

## Rejected Alternatives

- **Continue iterating partitioned SPIRE** (ADR-051/060 escalations):
  rejected — the Task 144 scan-fraction scaling (2.96% → 35.7% → 78.7%)
  shows the hedging cost is architectural, not a tuning residue.
- **BatANN-style baton passing** (arXiv:2512.09331): deferred per D4.
- **IVF-only distributed**: viable fallback recorded in the Task 146 lane;
  does not address the scan-fraction scaling at higher recall targets.
