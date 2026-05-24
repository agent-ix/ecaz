# Artifact Manifest: SPIRE Custom Scan UPDATE Datum JSON

- head SHA: `4e33e9642bbfbfcc10f1e6bd0cca96eb8bc24a63`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/221-spire-custom-scan-update-datum-json`
- timestamp: `2026-05-21T11:17:53Z`
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

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2538`
- `git grep -n "unsafe" HEAD -- src | wc -l`: `2536`
- `git grep -n "unsafe" HEAD^ -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `26`
- `git grep -n "unsafe" HEAD -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `24`
- `git grep -n "unsafe fn" HEAD^ -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `10`
- `git grep -n "unsafe fn" HEAD -- src/am/ec_spire/custom_scan/dml.rs | wc -l`: `9`

