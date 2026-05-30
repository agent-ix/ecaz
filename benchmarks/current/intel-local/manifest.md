# Current Intel Local Benchmarks

Status: pending refresh.

Standard suite: `crates/ecaz-cli/suites/current/intel-local.json`

This lane is the promoted current benchmark state for local Intel desktop or
workstation measurements. Populate it only from packeted benchmark evidence or
from a deliberate current-lane refresh that records:

- head SHA and branch
- source packet path
- host CPU, memory, OS, PostgreSQL version, and socket/port
- suite config path and SHA256
- `suite-manifest.json`, `results.jsonl`, `results-report.jsonl`, and raw logs
- cache policy and isolated one-index-per-table status
- claim class: local development evidence

Initial historical reference:

- `benchmarks/task-50-local-baseline/`
