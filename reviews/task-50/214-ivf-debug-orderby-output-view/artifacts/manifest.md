# Artifact Manifest: IVF Debug ORDER BY Output View

- head SHA: `c8c2a9f4ddabb3a39acffc02b8f097f8eef4a757`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/214-ivf-debug-orderby-output-view`
- timestamp: `2026-05-21T10:43:00Z`
- storage format / rerank mode: not applicable; code-only unsafe burn-down slice
- isolated one-index-per-table or shared-table surface: not applicable; no benchmark or corpus run

## Artifacts

### `rustfmt-check.log`

- command: `rustfmt --check src/am/ec_ivf/scan.rs`
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

### `cargo-test-ec-ivf-pg18-pgtest-no-run.log`

- command: `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: existing Hadamard test-helper dead-code warnings; `Finished test profile`; unit-test executable built.

## Unsafe Counts

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2555`
- `rg -n "unsafe" src | wc -l`: `2554`
- `git grep -n "unsafe" HEAD^ -- src/am/ec_ivf/scan.rs | wc -l`: `46`
- `git grep -n "unsafe" HEAD -- src/am/ec_ivf/scan.rs | wc -l`: `45`
- `git grep -n "unsafe fn" HEAD^ -- src/am/ec_ivf/scan.rs | wc -l`: `12`
- `rg -n "unsafe fn" src/am/ec_ivf/scan.rs | wc -l`: `12`

