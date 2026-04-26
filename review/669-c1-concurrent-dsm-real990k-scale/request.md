# Review Request: Concurrent DSM Real 990k Scale

## Summary

This packet records the first PG18 real-990k scale build for the current
source-scored concurrent DSM graph assembly path.

The fixture was prepared with the new chunked corpus tooling and loaded via the
chunked/resumable loader. The measured index build was then run through a
controlled SQL script that explicitly set the PG18 worker headroom and table
parallel-worker reloption before `CREATE INDEX`.

## Result

Real 990k source-scored build with `m = 16`, `ef_construction = 128`,
`build_source_column = source`, and requested 8 graph workers:

| corpus rows | requested workers | launched workers | CREATE INDEX time | graph_us | index size |
|---:|---:|---:|---:|---:|---:|
| 990,000 | 8 | 8 | `01:31:57.326` | `4656361521` | `1351688192` bytes |

Timing row:

- `heap_ingest_us = 598095013`
- `flush_total_us = 4892094823`
- `graph_us = 4656361521`
- `stage_us = 209065560`
- `write_us = 24934225`

## Interpretation

The build launched the intended 8 graph workers, so this is not a worker
headroom failure. At this scale, the current in-Postgres graph build path is
still very long: 990k rows took about 92 minutes even with 8 workers.

This supports keeping the current implementation as a correctness/baseline
surface while opening follow-up design work for an offline or staged bulk graph
builder with checkpointed image generation and a shorter PostgreSQL publish
step.

## Artifacts

- `artifacts/pg18_real990k_m16_w8_build.sql`
- `artifacts/pg18_real990k_m16_w8_build.log`
- `artifacts/load_real990k_chunked_m16.log`
- `artifacts/manifest.md`

## Notes

The chunked loader's automatic index build was intentionally terminated before
completion because it did not set the controlled worker/session knobs used for
measurement. The loaded corpus/query tables remained intact, and the measured
build was run afterward from packet-local SQL.
