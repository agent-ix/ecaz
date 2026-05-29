# SPIRE Phase 13e Suite Read Matrix Manifest

- head SHA: `b3a86fbe9134f31ad9ff44f215dc655320284a8b`
- task bucket: `reviews/task-30/973-spire-phase13e-suite-read-matrix`
- timestamp: `2026-05-25T18:58:38Z`
- lane: AWS suite configuration and canonical `ecaz bench suite` read matrix expansion
- fixture: `scripts/spire-aws/suite-{correctness,representative,stress}.json`
- storage format: AWS suite defaults, `ec_spire`
- rerank mode: default suite/runtime state
- surface isolation: AWS tier configs only; dry-run expansion, no database workload executed

## Artifacts

- `suite-correctness.json`, `suite-representative.json`, `suite-stress.json`
  - command: `jq --arg artifact_dir <packet-tier-artifacts> '.artifact_dir = $artifact_dir' scripts/spire-aws/suite-<tier>.json`
  - result: generated packet-local suite configs matching `scripts/spire-aws/bench.sh` behavior.
- `suite-dry-run-correctness.log`
  - command: `target/debug/ecaz --database postgres bench suite run --config reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-correctness.json --dry-run --manifest-output reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-manifest-correctness.json`
  - result: 3 selected steps; recall, latency, and `spire-pipeline --include-production-read-profile`, all with packet-local `--log-output`.
- `suite-dry-run-representative.log`
  - command: `target/debug/ecaz --database postgres bench suite run --config reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-representative.json --dry-run --manifest-output reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-manifest-representative.json`
  - result: 11 selected steps; includes k=10/k=100 production read profile rows and transport sweep rows for `auto`, `json_tuple_payload_v1`, and `pg_binary_attr_v1`.
- `suite-dry-run-stress.log`
  - command: `target/debug/ecaz --database postgres bench suite run --config reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-stress.json --dry-run --manifest-output reviews/task-30/973-spire-phase13e-suite-read-matrix/artifacts/suite-manifest-stress.json`
  - result: 3 selected steps; recall, latency, and stress production read profile row.
- `suite-manifest-correctness.json`, `suite-manifest-representative.json`, `suite-manifest-stress.json`
  - result: dry-run manifests include expanded commands and expected packet-local log artifacts.
- `suite-audit-correctness.log`, `suite-audit-representative.log`, `suite-audit-stress.log`
  - command: `target/debug/ecaz --database postgres bench suite audit --config <packet suite config>`
  - result: audits passed for 3, 11, and 3 steps respectively.
- `cargo-test-ecaz-cli-suite.log`
  - command: `cargo test -p ecaz-cli suite`
  - result: passed, 29 focused suite tests.
- `cargo-check-ecaz-cli.log`
  - command: `cargo check -p ecaz-cli`
  - result: passed with existing `LoadedDistributedPlacementConfig::path` dead-code warning.
- `cargo-build-ecaz-cli.log`
  - command: `cargo build -p ecaz-cli`
  - result: passed with existing `LoadedDistributedPlacementConfig::path` dead-code warning.
- `cargo-fmt-check.log`
  - command: `cargo fmt --all -- --check`
  - result: passed with existing stable-rust warnings about ignored unstable rustfmt options.
- `bash-n-spire-aws-bench.log`
  - command: `bash -n scripts/spire-aws/bench.sh`
  - result: passed.
