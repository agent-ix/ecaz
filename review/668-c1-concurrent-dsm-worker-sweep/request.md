# Review Request: Concurrent DSM Worker Sweep

## Summary

This packet records a PG18 real-50k worker sweep for the current default
concurrent DSM source-scored build path.

No runtime code changed. The packet adds the SQL used for the sweep and the raw
`ecaz-cli --log-output` log.

## Result

Real 50k source-scored builds with `m = 16`, `ef_construction = 128`, and
`build_source_column = source`:

| requested workers | launched workers | CREATE INDEX time | graph_us |
|---:|---:|---:|---:|
| 1 | 1 | `07:12.017` | `395621949` |
| 2 | 2 | `04:59.790` | `268137745` |
| 4 | 4 | `03:24.964` | `173200231` |
| 8 | 7 | `04:08.671` | `216938590` |

The best point on this fixture remains 4 workers. Requesting 8 workers launched
only 7 graph workers under the current PG18 cluster limits and regressed versus
4 workers.

## Artifacts

- `artifacts/pg18_concurrent_dsm_real50k_worker_sweep.sql`
- `artifacts/pg18_concurrent_dsm_real50k_worker_sweep.log`
- `artifacts/manifest.md`

## Notes

Packet 666 remains the serial-baseline source of truth:

- serial source-scored build: `30:15.962`
- serial `graph_us = 1784269081`

Against that baseline, the 4-worker sweep point is about `8.86x` faster by
CREATE INDEX wall-clock and about `10.30x` faster in `graph_us`.
