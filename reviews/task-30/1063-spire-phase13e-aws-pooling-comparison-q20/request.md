# Review Request

Pooled vs unpooled production-read-profile at q=20 on the still-up 1062 AWS cluster.

This packet captures the targeted comparison requested before waiting on the full q=1000 representative suite. Both probes used the preserved packet 1062 topology, the existing `ec_spire_aws_repr_1m` real corpus/index, `--queries-limit 20`, `--sweep 8,16,24,32`, `--top-k 10`, `--include-production-read-profile`, and `--include-recall`.

Artifacts:

- `artifacts/debug-production-read-profile-q20-pool-off.log`: pool disabled with `PGOPTIONS="-c ec_spire.remote_search_connection_pool_size=0"`.
- `artifacts/debug-production-read-profile-q20-pool-on.log`: pool enabled/default.
- `artifacts/pooling-q20-delta-summary.tsv`: three-row TSV summary of socket, connect, and total p50 deltas.
- `artifacts/manifest.md`: head SHA, cluster IDs, exact commands, and key result lines.

Result: pooling eliminates all per-dispatch socket opens and connect/TLS startup time on this AWS path. At q=20, socket opens drop from `53/60/60/60` to `0/0/0/0`, `connect_p50` drops from `19-20 ms` to `0 ms`, and production-read `total_p50` improves by `9-11 ms` across nprobe `8,16,24,32` with identical recall rows.
