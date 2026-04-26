# Review Request: Concurrent DSM W8 Headroom

## Summary

This packet records a focused PG18 real-50k 8-worker diagnostic after increasing
the PG18 cluster worker-process headroom.

No runtime code changed in this packet. It adds the SQL used for the diagnostic
and the raw `ecaz-cli --log-output` log.

## Result

With `max_worker_processes = 16`, `max_parallel_workers = 16`, and
`max_parallel_maintenance_workers = 8`, the real-50k source-scored build
launched all 8 graph workers:

| requested workers | launched workers | heap workers | graph workers | CREATE INDEX time | graph_us |
|---:|---:|---:|---:|---:|---:|
| 8 | 8 | 0 | 8 | `02:27.948` | `116850823` |

This supersedes the packet 668 8-worker data point for scale conclusions. The
old run was constrained by `max_worker_processes = 8`, launched only 7 graph
workers, and took `04:08.671`.

## Artifacts

- `artifacts/pg18_concurrent_dsm_real50k_w8_headroom.sql`
- `artifacts/pg18_concurrent_dsm_real50k_w8_headroom.log`
- `artifacts/manifest.md`

## Notes

Packet 668 remains useful for the 1/2/4-worker sweep and for showing that the
old 8-worker regression was a cluster-headroom artifact. With sufficient
headroom, 8 workers is now the best real-50k point measured so far.
