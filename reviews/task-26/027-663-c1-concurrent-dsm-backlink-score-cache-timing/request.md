# Review Request: Concurrent DSM Backlink Score Cache Timing

## Summary

Please review commit `2b656612842b364ff772eb0e6183801753d940dd`, which reuses the concurrent DSM query-score cache across backlink rewrites for the same backlink target.

The native build path already scopes backlink scoring by target. This change makes the concurrent DSM path do the same:

- pending backlink selections are already sorted by target node and layer
- the score cache now resets only when the target node changes
- repeated layer rewrites for the same target can reuse scores for the same candidate nodes

## Result

Packet 662 AVX accumulator timing:

- CREATE INDEX wall time: `204909.120 ms (03:24.909)`
- `graph_us = 172760957`

This packet after target-scoped backlink score reuse:

- CREATE INDEX wall time: `197371.287 ms (03:17.371)`
- `graph_us = 165691168`

Observed improvement:

- CREATE INDEX wall-clock speedup vs packet 662: about `1.04x`
- graph phase speedup vs packet 662: about `1.04x`
- CREATE INDEX wall-clock speedup vs packet 660: about `1.05x`
- graph phase speedup vs packet 660: about `1.06x`

This is the clearest post-AVX improvement so far. It reduces redundant source scoring in backlink rewrite work instead of only improving temporary allocation or the dot-product kernel.

Recall check on the same first-10-query real subset:

- serial baseline recall@10: `0.91`
- backlink-cache concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial baseline recall@100: `0.762`
- backlink-cache concurrent DSM recall@100: `0.77`
- recall@100 delta: `0.007999957`
- exact quantized recall@10: `1` for both
- backlink-cache concurrent DSM `graph_below_exact_queries`: `7`
- backlink-cache concurrent DSM `worst_exact_gap`: `2`

## Validation

- `cargo test -p ecaz build_parallel -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

`cargo fmt --check` still reports pre-existing unrelated formatting drift in `crates/ecaz-cli/src/commands/quant/feasibility.rs` and `src/quant/rabitq.rs`.

## Artifacts

- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.sql`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.log`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.sql`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.log`
- `artifacts/manifest.md`

## Notes

The next high-leverage area is likely reducing how often backlink rewrites need full source rescoring, or adding instrumentation to separate forward search scoring from backlink rewrite scoring. This packet suggests backlink rescoring is material enough to optimize directly.
