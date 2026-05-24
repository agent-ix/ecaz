# Artifact Manifest: SPIRE CustomScan Dead Helper Trim

- head SHA: `4516b714814374e6d20f06a94ae34eae08214681`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/170-spire-customscan-dead-helper-trim/`
- timestamp: `2026-05-21T05:33:32Z`
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
  - `31 src/am/ec_spire/custom_scan/plan_private.rs`
  - `22 src/am/ec_spire/custom_scan/cost_helpers.rs`

### `unsafe-ledger-after.jsonl`

- command: `make UNSAFE_LEDGER=reviews/task-50/170-spire-customscan-dead-helper-trim/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/170-spire-customscan-dead-helper-trim unsafe-ledger`
- result: generated
- key line: `wrote 1872 unsafe ledger rows to reviews/task-50/170-spire-customscan-dead-helper-trim/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-generate.log`

- command: same as above
- result: passed
- key line: `wrote 1872 unsafe ledger rows to reviews/task-50/170-spire-customscan-dead-helper-trim/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/170-spire-customscan-dead-helper-trim/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- result: passed
- key line: `ledger covers 1872 current unsafe rows`
