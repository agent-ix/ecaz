# Current AWS Graviton Benchmarks

Status: pending refresh.

Standard suite: `crates/ecaz-cli/suites/current/aws-graviton.json`

This lane is the promoted current benchmark state for AWS Graviton arm64
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

- `benchmarks/task55-aws-diskann-lowcost-config-audit/`
- `benchmarks/task55-aws-diskann-lowcost-optimized/`
- `benchmarks/task59-aws-diskann-final-graviton-suite/`
- `benchmarks/task61-aws-hnsw-graviton-baseline/`
