# Current AWS Intel Benchmarks

Status: pending refresh.

Standard suite: `crates/ecaz-cli/suites/current/aws-intel.json`

This lane is the promoted current benchmark state for AWS Intel x86_64
measurements. Populate it only from packeted benchmark evidence or from a
deliberate current-lane refresh that records:

- head SHA and branch
- source packet path
- EC2 instance type, CPU architecture, memory, AMI, region, and storage class
- PostgreSQL version, socket/port, and relevant parameter settings
- suite config path and SHA256
- `suite-manifest.json`, `results.jsonl`, `results-report.jsonl`, raw logs,
  and any S3 artifact URI
- cache policy and isolated one-index-per-table status
- claim class: benchmark-packet evidence unless a dedicated product benchmark
  packet explicitly promotes it

Initial historical references:

- `reviews/task-67/038-corrected-100k-simd-benchmark/`
- `benchmarks/aws-round-rabitq-ivf/`
