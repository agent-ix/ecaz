# Task 98 Closeout Matrix

| # | Criterion | Status |
|---|---|---|
| 1 | tiled_lut32 + int8_approx32 modules (SVE conditional on Phase A) | scalar+NEON(int8)/scalar(tiled) live; SVE skipped per Phase A data (ge32 ≤0.08%) — exactly the conditional the criterion anticipated |
| 2 | HNSW exact dispatch through batch method ≥32 | done — plus the partial-width path that actually carries this surface; includes the hot/cold cold-read fix that revived ALL exact-mode batching (incl. Task 87 FullLut) |
| 3 | Recall byte-equal | PASS at all six (mode × corpus) cells |
| 4 | Documented batch-width distribution per corpus | DONE and decisive: mean ~2.5–3, ge32 0.025–0.081% (packet 002 tables) |
| 5 | Scoring share where kernel fires | int8_approx full NEON coverage ~300 ns/cand; tiled scalar this phase; per the criterion, sub-1.5× cells are documented, not backed out |
| 6 | e2e measured, no regression | PASS (int8 100k improves: 5.76 vs 6.92 ms p50) |
| 7 | pg_test HNSW surfaces | deferred to Linux per the macOS dyld policy |
| 8 | Safety docs | PASS (# Safety on the int8 NEON impl) |

AVX2 variants: same Intel-lane question as Tasks 93/95; vpmaddubsw (int8)
named as the candidate strategy in packet 001's design.

Per criterion 4's purpose, the headline closeout fact: **HNSW frontier
batches essentially never reach 32 wide** — the block-kernel payoff on
this AM flows entirely through partial-width SIMD dispatch, and the
Task 99 profile should treat HNSW accordingly.
