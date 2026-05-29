# Artifact Manifest: SPIRE Phase 13e Pooling Gate Report

- head SHA: `7fb949baf515c99145960399964b3848f541333e`
- task bucket: `reviews/task-30/974-spire-phase13e-pooling-gate-report`
- lane: SPIRE Phase 13e evidence-gated connection pooling
- fixture: synthetic `ecaz bench suite` report fixture with `spire-pipeline` query metrics and production read profile rows
- storage format: not applicable; report-only fixture
- rerank mode: not applicable; report-only fixture
- timestamp: 2026-05-25T12:10:00-07:00
- isolated one-index-per-table vs shared-table surface: not applicable; no PostgreSQL workload was executed

## Artifacts

### `pooling-gate-spire-pipeline.log`

- command used: synthetic fixture input for report parsing
- purpose: contains representative `query_metrics` rows and production read profile rows that exercise both pooling decisions.
- key result lines:
  - `ecaz_spire_query_metrics fixture=synthetic-profile nprobe=8 latency_p95_ms=10.000`
  - `production_read_profile fixture=synthetic-profile nprobe=8 connect_tls_p50_ms=0.500 connect_tls_p95_ms=1.000`
  - `ecaz_spire_query_metrics fixture=synthetic-profile nprobe=16 latency_p95_ms=10.000`
  - `production_read_profile fixture=synthetic-profile nprobe=16 connect_tls_p50_ms=1.200 connect_tls_p95_ms=1.600`

### `pooling-gate-suite-manifest.json`

- command used: synthetic suite manifest for `ecaz bench suite report`
- purpose: points the suite report command at `pooling-gate-spire-pipeline.log`.
- key result lines:
  - `status: succeeded`
  - `step_type: spire-pipeline`

### `pooling-gate-report.md`

- command used: `target/debug/ecaz --database postgres bench suite report --manifest reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-suite-manifest.json --results-output reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-results.jsonl`
- purpose: generated report proving the pooling gate derives decisions from parsed suite evidence.
- key result lines:
  - `## SPIRE Connection Pooling Gate`
  - `| synthetic-profile | 8 | 0.500 | 1.000 | 10.000 | 0.1000 | pooling_not_justified |`
  - `| synthetic-profile | 16 | 1.200 | 1.600 | 10.000 | 0.1600 | pooling_candidate |`

### `pooling-gate-results.jsonl`

- command used: same `ecaz bench suite report` invocation as `pooling-gate-report.md`
- purpose: structured parsed rows emitted by the suite report command.

### `cargo-test-ecaz-cli-suite.log`

- command used: `cargo test -p ecaz-cli suite`
- purpose: focused suite runner unit coverage, including the pooling gate report test.
- key result lines:
  - `test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 118 filtered out`

### `cargo-check-ecaz-cli.log`

- command used: `cargo check -p ecaz-cli`
- purpose: compile validation for the CLI crate.
- key result lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s)`

### `cargo-build-ecaz-cli.log`

- command used: `cargo build -p ecaz-cli`
- purpose: build validation for the CLI binary used by the report command.
- key result lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s)`

### `cargo-fmt-check.log`

- command used: `cargo fmt --all -- --check`
- purpose: formatting validation after the suite report update.
- key result lines:
  - command exited successfully
