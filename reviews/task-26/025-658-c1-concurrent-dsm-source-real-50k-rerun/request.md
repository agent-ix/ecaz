# Review Request: Real 50k Source-Scored Concurrent DSM Rerun

## Summary

Please review this measurement packet for the source-scored concurrent DSM graph build introduced by commit `50290adca464f236eacd05c2ae1f6a6a2ae12639`.

The prior real-corpus packet 656 stopped at:

```text
ERROR: concurrent DSM graph assembly does not support source-scored builds yet
```

After the source-corpus DSM support, the same real-corpus shape now builds the sidecar source-scored index with 4 graph workers and matches the serial baseline on recall@10 for the 10-query real subset.

## Result

Fixture:

- prefix: `tqhnsw_real_50k_reloaded`
- corpus rows: 50,000
- query rows: 1,000
- evaluated subset: first 10 queries by id
- dimensions: 1536
- index options: `m = 16`, `ef_construction = 128`, `build_source_column = source`
- scan setting: `ec_hnsw.ef_search = 200`

Build timing:

- requested workers: 4
- workers launched: 4
- heap tuples: 50,000
- index tuples: 50,000
- `heap_ingest_us = 28202651`
- `graph_us = 401643816`
- `stage_us = 2099800`
- `write_us = 951416`
- serial and concurrent DSM index sizes: 68,280,320 bytes each

Recall comparison:

- serial recall@10: `0.91`
- concurrent DSM recall@10: `0.91`
- recall@10 delta: `0`
- serial recall@100: `0.762`
- concurrent DSM recall@100: `0.771`
- recall@100 delta: `0.009000003`
- exact quantized recall@10: `1` for both
- concurrent DSM `graph_below_exact_queries`: `7`
- concurrent DSM `worst_exact_gap`: `2`

## Artifacts

- `artifacts/pg18_concurrent_dsm_source_real_50k_rerun.sql`
- `artifacts/pg18_concurrent_dsm_source_real_50k_rerun.log`
- `artifacts/manifest.md`

## Notes

The fixture was loaded under a fresh prefix because the earlier local smoke had removed the `embedding` column from the original `tqhnsw_real_50k` scratch table while leaving the source/query rows. The fresh prefix avoided destructively dropping the old table.

This packet still shows the current performance issue clearly: the concurrent DSM source-scored graph phase is correct on this subset, but graph assembly took about 401.6 seconds. That is the next optimization target.
