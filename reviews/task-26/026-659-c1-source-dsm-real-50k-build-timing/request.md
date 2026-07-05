# Review Request: Real 50k Source-Scored Build Timing

## Summary

Please review this timing packet for the source-scored concurrent DSM graph build path.

This packet measures serial and concurrent DSM sidecar index builds in one PG18 SQL run on the same loaded real 50k fixture, with the same index options:

- `m = 16`
- `ef_construction = 128`
- `build_source_column = source`
- table `parallel_workers = 4`
- `max_parallel_maintenance_workers = 4`

## Result

Fixture:

- prefix: `tqhnsw_real_50k_reloaded`
- corpus rows: 50,000
- query rows: 1,000
- dimensions: 1536
- existing serial baseline index size: 68,280,320 bytes

Build timing:

| build path | workers launched | CREATE INDEX wall time | graph_us | index bytes |
| --- | ---: | ---: | ---: | ---: |
| serial source-scored | 0 | 30:15.962 | 1,784,269,081 | 68,280,320 |
| concurrent DSM source-scored | 4 | 7:11.269 | 399,932,406 | 68,280,320 |

Observed speedup:

- CREATE INDEX wall-clock speedup: about `4.21x`
- graph phase speedup: about `4.46x`

## Artifacts

- `artifacts/pg18_source_dsm_real_50k_build_timing.sql`
- `artifacts/pg18_source_dsm_real_50k_build_timing.log`
- `artifacts/manifest.md`

## Notes

This timing packet complements packet 658. Packet 658 established source-scored real 50k recall parity for the concurrent DSM sidecar; this packet isolates the build-time comparison against a same-run serial sidecar.

The performance path is now real but still not done: concurrent DSM cuts source-scored real 50k wall time from about 30 minutes to about 7 minutes, while graph assembly remains the dominant cost at about 400 seconds.
