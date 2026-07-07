# ec_distann: Global-Graph Architecture and Milestone Definitions

Companion design doc for ADR-085 and the FR-075..FR-083 / NFR-017..NFR-020 /
StR-008 spec batch (authored Task 161, branch `task-161-ec-distann-specs`).
This document is the normative home of the milestone definitions M0–M5 that
the specs and test matrix reference.

## Architecture summary

One global Vamana graph over all indexed vectors. No partitions, no routing
decision, no boundary replication. Records are lean (coarse search code +
adjacency + embedded neighbor codes, FR-076) with the full-precision vector
held once in a co-placed heap row for node-local exact rerank (ADR-085 D11,
FR-078/FR-079 — the `ec_diskann` coarse-in-index / exact-from-heap split,
sharded), hash-placed across nodes
(FR-078), built by sharded parallel Vamana builds stitched into one graph
(FR-077), and searched by a coordinator loop: local head-index descent
(FR-080) then H batched hop-rounds of per-node `ec_distann_expand_nodes`
calls (FR-079/FR-081), all under epoch lifecycle FR-082 with the D10
published-epoch mutation model. DML per FR-083 (tombstones + delta buffer
early; incremental distributed insert as the committed final milestone).

Reuse map (verified file:line) and the measured motivation (SPIRE
remediation verdict) live in ADR-085 and the program plan; key reuse:
`src/am/ec_diskann/vamana.rs` (build/prune, pure), `ec_diskann/scan.rs`
(beam loop shape), `am/common/quant_codec.rs` (neighbor codes),
`ec_spire/custom_scan` + `coordinator/remote_candidates/dispatch.rs`
(CustomScan + pooled transport, post-142), `ec_spire/meta/{epoch,
placement_directory}.rs` (adapted), `ec_spire/build/routing_plan.rs:132-250`
(closure overlap, repurposed to build-time), `ec_spire/build/top_graph.rs`
(in-memory Vamana for the head index).

## Milestone definitions (normative)

| ID | Name | Delivers | Exit criterion |
|----|------|----------|----------------|
| **M0** | Single-node parity | `src/am/ec_distann/` scaffold, record format (FR-076), head index (FR-080 degenerate single-shard), FR-081 loop with local expansion, monolithic build | pg_test recall parity with ec_diskann at 10k (±0.002); bench A/B 10k/50k vs ec_diskann ≤1.3× latency; D1 storage ratio, D3 C-sensitivity, D7 codec comparison measured (TC-037); **kill-check spike** (ADR-085 D2): recall-vs-H curve × measured per-round transport cost projects multinode p50 |
| **M1** | Stitch | Sharded closure-overlap build + stitch pass (FR-077), seed-deterministic | TC-038 property suite green; TC-039 stitched-vs-monolithic distinct_recall within 0.001 at 100k; duplication factor + stitch stats + peak-memory rows in epoch manifest |
| **M2** | Two-node read path | Hash placement (FR-078), `ec_distann_expand_nodes` (FR-079), remote FR-081 loop, epoch fingerprint validation subset of FR-082, publish hand-off | TC-040/TC-041: 2-node top-k identical to single-node build (same corpus/seed); per-query expansion ≤ BW×H asserted; measured 2-node vs 1-node latency delta (D4 baton-passing reopen trigger measurement) |
| **M3** | Lifecycle + faults | 3-worker build/publish/retire (FR-082 full), tombstone delete + delta-buffer insert (FR-083 early slices), distann multinode fixture + fault drills | TC-042 drill matrix green (incl. hop_round_failure_mid_beam, missing_node_record, placement_drift, mid-delete); epoch swap under load consistent; 50k multinode recall ≥ single-node − 0.001 |
| **M4** | Bench gate (program gate) | EC_DISTANN profile, `distann-pipeline` suite step kind + release-guard coverage, multinode storage summation, gate matrix | TC-044: pre-registered four-way table (ec_distann / IVF / HNSW / best-SPIRE) at 10k/50k/100k per NFR-017 matched-recall rule; NFR-018 ratio rows; NFR-019 min-BW×H row; informational netem run; promote/iterate/shelve verdict written into ADR-085 status. Prerequisite: task-138 `distinct_recall` + task-146 anchors merged |
| **M5** | Incremental insert (committed) | FR-083 full: write endpoint, distributed self-insertion, back-edge RMW + re-prune, collision error, UPDATE semantics | TC-043 + FR-083-AC-4 bench cell: insert-then-query distinct_recall parity with fresh rebuild; mid-insert + concurrency drills green |

Milestone→task mapping: M0=162, M1=163, M2=164, M3=165, M4=166, M5=167;
spec authoring = 161 (this lane).

## Design invariants worth restating

- Results come only from expanded records; BW×H is a hard per-attempt cap
  (max two attempts via the FR-082 epoch-mismatch restart).
- Within a published epoch nothing is physically reclaimed; vacuum reclaim +
  edge repair are epoch-build operations (this resolves the
  reclaimed-neighbor/missing-record contradiction found in spec review).
- Placement is load-balance-only; every placement disagreement is an error,
  never a silent miss. Placement co-locates each record's full-precision heap
  row on the same owning node (ADR-085 D11), so exact rerank is node-local.
- The expand-response wire contract is independent of the D1 record layout.
- `records read == nodes expanded == nodes exact-reranked == nodes
  materialized` (all ≤ BW×H) — the equality that makes co-placed heap rerank
  (D11) affordable where SPIRE's wide-leaf scan-fraction forced an
  index-resident code.

## Open items tracked to milestones

- D1/D7 arithmetic sits at NFR-018's 4.0× threshold at R=32 defaults once
  D11 removes the inline vector (~5.0× → ~4.0×) → M0 measures whether the R×
  neighbor-code block still needs lower R / smaller codes / fallback layout.
- H×RTT floor → M0 kill-check spike projects it; M2 measures it; D4 reopen
  trigger (hop RTT ≥ 50% of multinode p50) reopens baton passing.
- NFR-018 multinode storage summation is a suite-runner extension landing as
  its own commit before M4.
