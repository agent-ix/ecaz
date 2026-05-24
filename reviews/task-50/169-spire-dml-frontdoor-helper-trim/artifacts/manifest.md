# Artifact Manifest: SPIRE DML Frontdoor Helper Trim

- head SHA: `8aea3f5934623ab58b657909b3cba9db10b5d5c2`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/169-spire-dml-frontdoor-helper-trim/`
- timestamp: `2026-05-21T05:30:24Z`
- lane: Task 50 unsafe burndown
- fixture: local PG18/bench compile and unsafe ledger validation
- storage format: source-only validation, no benchmark storage fixture
- rerank mode: not applicable
- surface isolation: not applicable, no index/table benchmark run

## Artifacts

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo]`
  - warning is the existing unused-import warning in `src/am/mod.rs`

### `git-diff-check.log`

- command: `git diff --check HEAD~1..HEAD`
- result: passed
- key lines: no output

### `unsafe-block-count.log`

- command: `make unsafe-block-count`
- result: passed
- key lines:
  - `59 src/am/ec_spire/dml_frontdoor/mod.rs`
  - `2 src/am/ec_spire/dml_frontdoor/tests.rs`

### `unsafe-ledger-after.jsonl`

- command: `make UNSAFE_LEDGER=reviews/task-50/169-spire-dml-frontdoor-helper-trim/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/169-spire-dml-frontdoor-helper-trim unsafe-ledger`
- result: generated
- key line: `wrote 1876 unsafe ledger rows to reviews/task-50/169-spire-dml-frontdoor-helper-trim/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-generate.log`

- command: same as above
- result: passed
- key line: `wrote 1876 unsafe ledger rows to reviews/task-50/169-spire-dml-frontdoor-helper-trim/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/169-spire-dml-frontdoor-helper-trim/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- result: passed
- key line: `ledger covers 1876 current unsafe rows`
