# Artifact Manifest

Packet: `review/676-c1-task29-isolated-real10k-baseline`

Head SHA: `9d4d10ec2e5c54e9e0b79705f92e6fd13e809e82`

Timestamp: `2026-04-29T15:05:50-07:00`

Lane: Task 29 DiskANN initial tuning, local PG18 only.

Fixture: real-10k, 10000 corpus rows, 200 query rows, 1536 dimensions,
`bits=4`, `seed=42`.

Surfaces:

- DiskANN isolated: `task29_diskann_real10k_corpus`,
  `task29_diskann_real10k_queries`, one index
  `task29_diskann_real10k_idx`.
- HNSW isolated reference: `task29_hnsw_real10k_corpus`,
  `task29_hnsw_real10k_queries`, one index
  `task29_hnsw_real10k_m16_idx`.

Storage format: default `ecvector` encoding. Rerank mode: not applicable for
DiskANN/HNSW in this run.

Cache state: no cache flush. `explain-diskann-isolated-q1.log` ran before the
full sweeps and reported `shared hit=186 read=736`; recall and latency sweeps
then ran in the same local PG18 cluster and should be treated as warm-cache
local measurements.

## Artifacts

### `drop-isolated-prefixes.log`

Command:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "DROP TABLE IF EXISTS task29_diskann_real10k_corpus CASCADE; DROP TABLE IF EXISTS task29_diskann_real10k_queries CASCADE; DROP TABLE IF EXISTS task29_hnsw_real10k_corpus CASCADE; DROP TABLE IF EXISTS task29_hnsw_real10k_queries CASCADE;" --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/drop-isolated-prefixes.log
```

Key result: all four isolated relations were absent or dropped cleanly.

### `load-diskann-isolated.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/load-diskann-isolated.log corpus load --prefix task29_diskann_real10k --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch
```

Key result lines:

```text
copied corpus table task29_diskann_real10k_corpus in 9.70s
encoded corpus table task29_diskann_real10k_corpus in 4.21s
copied queries table task29_diskann_real10k_queries in 213.61ms
built task29_diskann_real10k_idx in 491.05s
completed prefix task29_diskann_real10k in 520.66s
```

### `load-hnsw-isolated.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/load-hnsw-isolated.log corpus load --prefix task29_hnsw_real10k --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_hnsw --m 16 --allow-manifest-mismatch
```

Key result lines:

```text
copied corpus table task29_hnsw_real10k_corpus in 9.67s
encoded corpus table task29_hnsw_real10k_corpus in 4.69s
copied queries table task29_hnsw_real10k_queries in 265.12ms
built task29_hnsw_real10k_m16_idx in 89.14s
completed prefix task29_hnsw_real10k in 119.72s
```

### `diskann-build-activity.log`

Command:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "SELECT now(), state, wait_event_type, wait_event, left(query, 160) AS query FROM pg_stat_activity WHERE datname = 'task29_diskann_baseline' ORDER BY backend_start;" --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/diskann-build-activity.log
```

Key result: DiskANN build was actively running inside `CREATE INDEX
task29_diskann_real10k_idx`.

### `explain-diskann-isolated-q1.sql`

Packet-local SQL used by `explain-diskann-isolated-q1.log`.

### `explain-diskann-isolated-q1.log`

Command:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --file review/676-c1-task29-isolated-real10k-baseline/artifacts/explain-diskann-isolated-q1.sql --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/explain-diskann-isolated-q1.log
```

Key result lines:

```text
Index Scan using task29_diskann_real10k_idx on task29_diskann_real10k_corpus
Buffers: shared hit=186 read=736
Execution Time: 68.055 ms
```

### `truth-k10.json`

Command that created it:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/recall-diskann-isolated-cli.log bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --truth-cache-file review/676-c1-task29-isolated-real10k-baseline/artifacts/truth-k10.json --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/recall-diskann-isolated-table.log
```

Key result: exact ground truth for 200 queries was computed in `3.47s`.

### `recall-diskann-isolated-cli.log`

Command: same as the `truth-k10.json` command.

Key result: fetched 200 queries and 10000 corpus rows, computed and wrote
`truth-k10.json`.

### `recall-diskann-isolated-table.log`

Command: same as `recall-diskann-isolated-cli.log` with `--log-output` pointing
to this artifact.

Key result:

```text
64  0.9280  0.9959  70.96 ms
128 0.9310  0.9966  74.01 ms
200 0.9315  0.9966  84.70 ms
400 0.9315  0.9966  126.73 ms
800 0.9315  0.9966  268.90 ms
```

### `recall-hnsw-isolated-cli.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/recall-hnsw-isolated-cli.log bench recall --prefix task29_hnsw_real10k --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --truth-cache-file review/676-c1-task29-isolated-real10k-baseline/artifacts/truth-k10.json --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/recall-hnsw-isolated-table.log
```

Key result: reused `truth-k10.json`.

### `recall-hnsw-isolated-table.log`

Command: same as `recall-hnsw-isolated-cli.log` with `--log-output` pointing to
this artifact.

Key result:

```text
64  0.9305  0.9814  18.63 ms
128 0.9645  0.9967  28.04 ms
200 0.9700  0.9993  37.42 ms
400 0.9710  0.9994  63.30 ms
800 0.9720  0.9995  119.12 ms
```

### `latency-diskann-isolated-cli.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/latency-diskann-isolated-cli.log bench latency --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/latency-diskann-isolated-table.log
```

Key result: table written to `latency-diskann-isolated-table.log`.

### `latency-diskann-isolated-table.log`

Command: same as `latency-diskann-isolated-cli.log` with `--log-output`
pointing to this artifact.

Key result:

```text
64  p50=61.7 ms  p95=65.4 ms   p99=68.0 ms   HWM=82632 KiB
128 p50=72.5 ms  p95=77.9 ms   p99=81.9 ms   HWM=83752 KiB
200 p50=84.1 ms  p95=95.6 ms   p99=104.9 ms  HWM=83432 KiB
400 p50=125.7 ms p95=142.4 ms  p99=148.8 ms  HWM=84072 KiB
800 p50=267.0 ms p95=301.3 ms  p99=316.2 ms  HWM=84232 KiB
```

### `latency-hnsw-isolated-cli.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/latency-hnsw-isolated-cli.log bench latency --prefix task29_hnsw_real10k --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/676-c1-task29-isolated-real10k-baseline/artifacts/latency-hnsw-isolated-table.log
```

Key result: table written to `latency-hnsw-isolated-table.log`.

### `latency-hnsw-isolated-table.log`

Command: same as `latency-hnsw-isolated-cli.log` with `--log-output` pointing
to this artifact.

Key result:

```text
64  p50=17.9 ms  p95=25.1 ms   p99=29.6 ms   HWM=77352 KiB
128 p50=26.9 ms  p95=36.1 ms   p99=44.4 ms   HWM=78152 KiB
200 p50=35.7 ms  p95=43.4 ms   p99=51.2 ms   HWM=78792 KiB
400 p50=62.3 ms  p95=70.2 ms   p99=76.0 ms   HWM=79112 KiB
800 p50=118.9 ms p95=144.2 ms  p99=158.6 ms  HWM=80128 KiB
```

### `storage-diskann-isolated.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/storage-diskann-isolated.log bench storage --prefix task29_diskann_real10k
```

Key result:

```text
task29_diskann_real10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B
total 164.5 MiB
```

### `storage-hnsw-isolated.log`

Command:

```sh
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/676-c1-task29-isolated-real10k-baseline/artifacts/storage-hnsw-isolated.log bench storage --prefix task29_hnsw_real10k
```

Key result:

```text
task29_hnsw_real10k_m16_idx  ec_hnsw  {m=16,ef_construction=128,build_source_column=source}  13.0 MiB  1366.4 B
total 172.9 MiB
```
