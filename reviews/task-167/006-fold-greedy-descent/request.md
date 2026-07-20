# Task 167 (M5) — packet 006: incremental fold reaches REBUILD PARITY

Coder packet. Resolves the packet-167/005 finding that the incremental fold was
not recall-viable. `graph_insert_record` now finds the new node's neighbors via
the proven FR-081 scan search (`collect_distann_hits`) instead of the fixed
head-sample region.

## Result (real DBpedia, release .so) — fold recall@10 now == full rebuild

| scale | fold (tk=200) | full rebuild (026) |
| ----- | ------------- | ------------------ |
| 10k   | 1.0000 | 1.0000 |
| 50k   | 0.9965 | 0.9950 |
| 100k  | 0.9905 | 0.9925 |

At every scale + sweep point the fold matches or beats full rebuild (within
±0.008 at worst). This resolves Task 167's distinct_recall-parity acceptance
criterion (004-P2). Evolution: old head-sample-only fold 0.878/0.691 (10k/50k
tk=200) → greedy descent 0.993/0.828 → scan-search 1.0/0.9965.

## Evidence

- `artifacts/manifest.md` — full matrix + code description.
- `artifacts/fold-recall-{10k,50k}-greedy.log`, `fold-recall-100k-scansearch.log`.
- Code: commit `f70691402` (collect_distann_hits candidate search).

## Remaining M5 (unchanged)

- 004-P2 perf: O(N)-per-row directory rewrite (recall solved; throughput is not).
- 004-P1: distributed aminsert self-insertion (unbuilt).
- single-txn CREATE INDEX+INSERT+fold snapshot/heap-open error (normal path works).
