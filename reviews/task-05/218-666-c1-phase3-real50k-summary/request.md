# Review Request: Phase 3 Real 50k Summary

## Summary

This packet consolidates the Phase 3 real-50k recall and build-time evidence for Task 26.

It does not introduce new code. It gathers the raw logs from the packet chain into one review packet so the default-switch decision has a single source of truth.

## Result

Recall parity on the first-10-query real 50k subset:

- serial baseline recall@10: `0.91`
- concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial baseline recall@100: `0.762`
- current best concurrent DSM recall@100: `0.77`
- current best recall@100 delta: `0.007999957`
- exact quantized recall@10: `1` for both
- graph below exact queries: `7`
- worst exact gap: `2`

Build-time comparison:

- serial source-scored build: `1815962.457 ms (30:15.962)`
- serial source-scored `graph_us = 1784269081`
- current best concurrent DSM source-scored build: `197371.287 ms (03:17.371)`
- current best concurrent DSM `graph_us = 165691168`

Observed speedup:

- CREATE INDEX wall-clock speedup vs serial: about `9.20x`
- graph phase speedup vs serial: about `10.77x`

## Artifacts

- `artifacts/pg18_concurrent_dsm_source_real_50k_rerun.sql`
- `artifacts/pg18_concurrent_dsm_source_real_50k_rerun.log`
- `artifacts/pg18_source_dsm_real_50k_build_timing.sql`
- `artifacts/pg18_source_dsm_real_50k_build_timing.log`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.sql`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_timing.log`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.sql`
- `artifacts/pg18_source_score_backlink_cache_concurrent_50k_recall.log`
- `artifacts/manifest.md`

## Notes

This packet is intended to close Phase 3 of Task 26 for the real 50k surface:

- recall is faithful at recall@10
- the build-time win is graph-phase material, not just heap-ingest overhead
- packet 665 separately flips concurrent DSM graph assembly on by default for eligible parallel builds

Phase 5 scale curves remain separate work.
