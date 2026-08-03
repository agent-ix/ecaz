# Suite audit and dry-run

Head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

Command:

`target/debug/ecaz bench suite audit --config reviews/task-206/002-100k-sweep/artifacts/task206-100k-sweep.json`

Result: `audit passed: 1 steps`

Command:

`target/debug/ecaz bench suite run --config reviews/task-206/002-100k-sweep/artifacts/task206-100k-sweep.json --dry-run`

Key expansion result: one `distann-multicluster local-multinode-pg18`
command with nine `--benchmark-seed-variant` arms:

- `bw32-h4`, `bw32-h5`, `bw32-h8`
- `bw64-h4`, `bw64-h5`, `bw64-h8`
- `bw128-h4`, `bw128-h5`, `bw128-h8`

The emitted command also contains `--build-shards 1`, `--top-k 200`,
`--head-index-cap 4096`, `--corpus-prefix ec_real_100k`, and the external
archived corpus directory. This is configuration evidence only; no benchmark
result rows were produced.
