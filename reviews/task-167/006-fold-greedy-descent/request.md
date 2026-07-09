# Task 167 (M5) — packet 006: fold redesign (greedy graph descent) closes the recall gap

Coder packet. Fixes the packet-167/005 finding that the incremental fold was not
recall-viable. `graph_insert_record` now runs a greedy best-first graph descent
(code-based traversal, exact vectors only for the final robust_prune pool)
instead of selecting neighbors from the fixed head-sample region.

## Result (10k, real DBpedia) — recall@10 fold vs old fold vs full rebuild

| sweep | old fold (005) | greedy fold (this) | full rebuild (026) |
| ----- | -------------- | ------------------ | ------------------ |
| 64  | 0.847 | **0.974** | 0.9995 |
| 200 | 0.878 | **0.993** | 1.0000 |

The greedy descent nearly closes the fold-recall gap at 10k (gap to full rebuild
0.122 → 0.007 at tk=200). 50k confirmation (the scale where the old fold collapsed
to 0.691) is running and will be appended to `artifacts/manifest.md`.

## Evidence

- `artifacts/manifest.md` — full table + code description.
- `artifacts/fold-recall-10k-greedy.log` (+ `-50k-greedy.log` when it lands).
- Code: commit `18d1c18fa` (src/am/ec_distann/insert.rs greedy_insert_candidates).

## Status

This materially advances Task 167's distinct_recall-parity acceptance criterion
(fold is now recall-competitive at 10k). Remaining M5 items: 50k/100k confirmation,
the single-txn snapshot/heap-open hardening, O(N)-per-insert directory rewrite
(004-P2), and the distributed aminsert self-insertion surface (004-P1).
