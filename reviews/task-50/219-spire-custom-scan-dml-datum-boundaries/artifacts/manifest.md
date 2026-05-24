# Artifact Manifest: SPIRE Custom Scan DML Datum Boundaries

- head SHA: `339f19daa254e2737f541e52169b501ce3593aa9`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/219-spire-custom-scan-dml-datum-boundaries`
- timestamp: `2026-05-21T11:10:25Z`
- storage format / rerank mode: not applicable; code-only unsafe burn-down slice
- isolated one-index-per-table or shared-table surface: not applicable; no benchmark or corpus run

## Artifacts

### `rustfmt-check.log`

- command: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs`
- result: passed
- key lines: stable-channel warnings for `imports_granularity` and `group_imports`; no formatting diff.

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: no output.

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: existing unused-import warning in `src/am/mod.rs`; `Finished dev profile`.

### `cargo-test-custom-scan-pg18-pgtest-no-run.log`

- command: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: existing Hadamard test-helper dead-code warnings; `Finished test profile`; unit-test executable built.

## Unsafe Counts

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2544`
- `git grep -n "unsafe" HEAD -- src | wc -l`: `2540`
- `git grep -n "unsafe" HEAD^ -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `32`
- `git grep -n "unsafe" HEAD -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `28`
- `git grep -n "unsafe fn" HEAD^ -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `13`
- `git grep -n "unsafe fn" HEAD -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `11`

