# Re-Scorable Artifact Inventory (Task 138 Phase 1)

Question: which historical packets contain per-query returned-ID evidence that
can be re-scored with `distinct_recall@k` without re-running?

Method: `find reviews benchmarks -name "*identity*.jsonl"` over the full
repo at the audit head, plus a sweep for prediction dumps
(`*predictions*.json`).

## Re-scorable as-is (per-query returned IDs on disk)

| Packet | Artifacts | Lane / shape | Status |
| --- | --- | --- | --- |
| `reviews/task-131/027-phase3-increment-a-ab/` | `artifacts/{10k,50k}-n128-b4/bench-suite/production-read-k10-threshold-{off,on}-default-identity.jsonl` | local multi-instance, n128/b4, nprobe=96, k=10, 200q (10k) / 1000q (50k) | re-scored in this packet (`artifacts/rescore/`) |

This is the only SPIRE multi-instance returned-ID evidence in the repo. The
`spire_result_identity` JSONL format was introduced by Task 131 packet 027;
no earlier SPIRE packet emitted per-query returned IDs.

Non-SPIRE note: `reviews/task-47/004-cross-am-consistency-metrics/` contains
per-query prediction dumps for ec_hnsw / ec_diskann single-instance cross-AM
consistency. Single-instance scans dedupe by vec_id within one node, so those
lanes are not exposed to the cross-node duplicate defect and are out of audit
scope.

## Re-runnable cheaply (no returned-ID artifacts; shape and configs preserved)

| Prior evidence | Shape | Re-run cell in this packet |
| --- | --- | --- |
| Task 123 packets 009/019/020, Task 131 packet 024 multi-instance baselines | n1024/b2, nprobe=64, k=10 | `artifacts/{10k,50k}-n1024-b2/` (200q, this packet) |
| Task 131 packet 027 (as cross-check of the re-score path) | n128/b4, nprobe=96, k=10 | not re-run; identity artifacts re-scored directly |

## Unrecoverable (returned IDs never captured; exact-run re-scoring impossible)

- Task 121 Phase 2 DOE local single-instance recall cells
  (`reviews/task-121/011..018`): per-query predictions were not persisted.
  However, these ran on the **local single-instance lane**, where the scan
  dedupes boundary replicas by vec_id inside one index, so their recall
  numbers are not exposed to the cross-node duplicate defect (see
  `src/tests/build.rs` local-dedupe coverage and ADR-083). They are labeled
  "not duplicate-exposed" rather than re-scored.
- Task 123 multi-instance packets before identity emission (packets 009-021):
  duplicate-exposed but returned IDs were never captured. The equivalent
  shapes are covered by the fresh n1024/b2 and rescored n128/b4 cells above;
  the historical latency numbers remain valid (latency is identity-agnostic),
  while their recall figures must be read as duplicate-tolerant.
