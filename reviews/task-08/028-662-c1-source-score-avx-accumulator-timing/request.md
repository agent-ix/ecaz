# Review Request: AVX Source-Score Accumulator Timing

## Summary

Please review commit `1802d707339a24785ee1cea46688a3e5b50c2056`, which changes the source-vector build inner-product AVX2+FMA path from one vector accumulator to four independent accumulators.

The goal is to reduce the dependency chain in the 1536-dimensional source dot product used by source-scored native and concurrent DSM graph builds. The SQL-facing rerank path still uses the existing sequential f32 accumulation order.

## Result

Packet 660 post-AVX concurrent DSM source-scored timing:

- CREATE INDEX wall time: `207594.528 ms (03:27.595)`
- `graph_us = 174810922`

Packet 661 scratch-buffer timing:

- CREATE INDEX wall time: `209737.611 ms (03:29.738)`
- `graph_us = 176719376`

This packet after AVX accumulator unroll:

- CREATE INDEX wall time: `204909.120 ms (03:24.909)`
- `graph_us = 172760957`

Observed improvement:

- CREATE INDEX wall-clock speedup vs packet 660: about `1.01x`
- graph phase speedup vs packet 660: about `1.01x`
- CREATE INDEX wall-clock speedup vs packet 661: about `1.02x`
- graph phase speedup vs packet 661: about `1.02x`

This is a real but small improvement. It does not change the conclusion that the dominant remaining cost is the amount of source-scored graph/backlink work, not just the per-dot-product kernel.

Recall check on the same first-10-query real subset:

- serial baseline recall@10: `0.91`
- AVX accumulator concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial baseline recall@100: `0.762`
- AVX accumulator concurrent DSM recall@100: `0.774`
- recall@100 delta: `0.011999965`
- exact quantized recall@10: `1` for both
- AVX accumulator concurrent DSM `graph_below_exact_queries`: `7`
- AVX accumulator concurrent DSM `worst_exact_gap`: `2`

## Validation

- `cargo test inner_product -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

`cargo fmt --check` still reports pre-existing unrelated formatting drift in `crates/ecaz-cli/src/commands/quant/feasibility.rs` and `src/quant/rabitq.rs`.

## Artifacts

- `artifacts/pg18_source_score_avx_accum_concurrent_50k_timing.sql`
- `artifacts/pg18_source_score_avx_accum_concurrent_50k_timing.log`
- `artifacts/pg18_source_score_avx_accum_concurrent_50k_recall.sql`
- `artifacts/pg18_source_score_avx_accum_concurrent_50k_recall.log`
- `artifacts/manifest.md`

## Notes

This slice keeps the fast path bounded to the build-only source scorer. It is worth keeping because it is simple and validated, but it is not enough to materially change the remaining performance profile.
