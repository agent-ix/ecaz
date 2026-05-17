# Review Request: AVX Source-Score Build Timing

## Summary

Please review commit `d9b79567c9402e254ff193f7bb4abb5995cb2c26`, which routes source-scored build graph dot products through a shared source inner-product helper with an x86 AVX2+FMA fast path and scalar fallback.

The SQL-facing rerank score path keeps its previous sequential f32 accumulation order to preserve exact regression-test expectations. This change is scoped to build graph scoring:

- native source graph build scoring
- concurrent DSM source graph build scoring

## Result

Packet 659 pre-change concurrent DSM source-scored timing:

- CREATE INDEX wall time: `431268.704 ms (07:11.269)`
- `graph_us = 399932406`

This packet post-change concurrent DSM source-scored timing:

- CREATE INDEX wall time: `207594.528 ms (03:27.595)`
- `graph_us = 174810922`

Observed improvement:

- CREATE INDEX wall-clock speedup vs packet 659 concurrent DSM: about `2.08x`
- graph phase speedup vs packet 659 concurrent DSM: about `2.29x`
- CREATE INDEX wall-clock speedup vs packet 659 serial source-scored build: about `8.75x`

Recall check on the same first-10-query real subset:

- serial baseline recall@10: `0.91`
- AVX concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial baseline recall@100: `0.762`
- AVX concurrent DSM recall@100: `0.772`
- recall@100 delta: `0.00999999`
- exact quantized recall@10: `1` for both
- AVX concurrent DSM `graph_below_exact_queries`: `7`
- AVX concurrent DSM `worst_exact_gap`: `2`

## Validation

- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

`cargo fmt --check` still reports pre-existing unrelated formatting drift in `crates/ecaz-cli/src/commands/quant/feasibility.rs` and `src/quant/rabitq.rs`.

## Artifacts

- `artifacts/pg18_source_score_avx_concurrent_50k_timing.sql`
- `artifacts/pg18_source_score_avx_concurrent_50k_timing.log`
- `artifacts/pg18_source_score_avx_concurrent_50k_recall.sql`
- `artifacts/pg18_source_score_avx_concurrent_50k_recall.log`
- `artifacts/manifest.md`

## Notes

This is a real performance win, not a threshold tweak. It attacks the dominant remaining graph phase by reducing the cost of the 1536-float source dot products used during source-scored graph construction.

The graph phase is still the dominant cost at about 175 seconds on real 50k, so the next likely target is reducing the number of exact source scores or reducing graph search/backlink work, not just making individual scores faster.
