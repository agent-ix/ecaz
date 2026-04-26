# Review Request: Concurrent DSM Successor Scratch-Buffer Measurement

## Summary

Please review commit `28c64c699c639bad8ce040c9ee7ecac0fb324982`, which reuses the concurrent DSM layer-search successor neighbor-index buffer instead of allocating a fresh `Vec<u32>` for every successor expansion.

This is a narrow allocation cleanup in the concurrent DSM graph-search path:

- `EcHnswConcurrentDsmLayerSearchScratch` now owns `neighbor_idxs`
- each expansion clears and reuses that buffer before loading successor candidates
- graph behavior should be unchanged

## Result

Packet 660 post-AVX concurrent DSM source-scored timing:

- CREATE INDEX wall time: `207594.528 ms (03:27.595)`
- `graph_us = 174810922`

This packet after successor scratch-buffer reuse:

- CREATE INDEX wall time: `209737.611 ms (03:29.738)`
- `graph_us = 176719376`

Observed impact:

- CREATE INDEX wall-clock was about `1.01x` slower than packet 660
- graph phase was about `1.01x` slower than packet 660
- this change did not produce a measurable performance win at real 50k scale

Recall check on the same first-10-query real subset:

- serial baseline recall@10: `0.91`
- scratch-buffer concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial baseline recall@100: `0.762`
- scratch-buffer concurrent DSM recall@100: `0.768`
- recall@100 delta: `0.0059999824`
- exact quantized recall@10: `1` for both
- scratch-buffer concurrent DSM `graph_below_exact_queries`: `7`
- scratch-buffer concurrent DSM `worst_exact_gap`: `2`

## Validation

- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

`cargo fmt --check` still reports pre-existing unrelated formatting drift in `crates/ecaz-cli/src/commands/quant/feasibility.rs` and `src/quant/rabitq.rs`.

## Artifacts

- `artifacts/pg18_source_score_scratchbuf_concurrent_50k_timing.sql`
- `artifacts/pg18_source_score_scratchbuf_concurrent_50k_timing.log`
- `artifacts/pg18_source_score_scratchbuf_concurrent_50k_recall.sql`
- `artifacts/pg18_source_score_scratchbuf_concurrent_50k_recall.log`
- `artifacts/manifest.md`

## Notes

This confirms that removing this per-expansion allocation is not a meaningful next performance lever after the AVX source-score kernel. The dominant cost remains source-scored graph search/backlink work, not allocation of the temporary successor neighbor list.
