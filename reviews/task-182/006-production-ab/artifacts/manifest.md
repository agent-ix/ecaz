# Task 182 production A/B manifest

- Status: release build installed; measurements pending
- Configuration checkpoint: `8769d57834170dcdc586fa8ac85b99e50b656bd8`
- Task bucket / packet: `reviews/task-182/006-production-ab/`
- Suite config: `production-ab-suite.json`
- Matrix: current production vs trained production vs owner-oracle diagnostic
  at 10k / 50k / 100k
- Fixture: three physical owners, fresh generation per step
- Query shape: 200 held-out queries, top-10, BW4/H100
- Latency: 50 warm iterations after 10 warmups, concurrency 1
- Storage: physical generation/control/source/single plus persisted and cached
  head estimates
- Neighbor/rerank: RaBitQ neighbor scoring; existing exact final rerank
- Training: rows 201–400 from each declared staged query file (10k uses the
  100k query file, matching the reviewed Task 181 disjoint slice)
- Corpus/query TSVs and truth caches are not committed

## Dry-run validation

- Command: `cargo run -p ecaz-cli -- bench suite run --config reviews/task-182/006-production-ab/artifacts/production-ab-suite.json --dry-run`
- Timestamp: 2026-07-16 (America/Los_Angeles)
- Result: success; nine selected steps, all with status `dry-run`
- Generated manifest: `run/suite-manifest.json`
- Expanded steps: `current`, `trained`, and `oracle` at 10k, 50k, and 100k

## Measurement build

- Build / install SHA: `f02cf58a0` (full SHA will be attested by every suite
  node and recorded with the results)
- Extension command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
- Extension result: success; release library installed and SQL entities
  generated for PG18
- Extension log: `implementation-install.log`
- Runner command: `cargo build --release -p ecaz-cli`
- Runner result: success; one pre-existing unused-field warning
- Runner log: `cli-release-build.log`

Head SHA, command, timestamp, isolation, suite-manifest path, results path, and
key result lines will be recorded after execution. No number is claimed yet.
