# Current M5 Local Benchmarks

Status: pending refresh.

Standard suite: `crates/ecaz-cli/suites/current/m5-local.json`

This lane is the promoted current benchmark state for Apple Silicon M5 local
development measurements. Populate it only from packeted benchmark evidence or
from a deliberate current-lane refresh that records:

- head SHA and branch
- source packet path
- host CPU, memory, OS, PostgreSQL version, and socket/port
- suite config path and SHA256
- `suite-manifest.json`, `results.jsonl`, `results-report.jsonl`, and raw logs
- cache policy and isolated one-index-per-table status
- claim class: local development evidence

Initial historical references:

- `benchmarks/task-50-m5-hnsw-baseline/`
- `benchmarks/task-55-m5-diskann-baseline/`
- `benchmarks/task60-diskann-rabitq-format/`
- `benchmarks/task63-hnsw-rabitq-format/`
