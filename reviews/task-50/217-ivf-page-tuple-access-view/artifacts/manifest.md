# Artifact Manifest: IVF Page Tuple Access View

- head SHA: `f343382b19367f768b5da5afb8b3bf529c1d228f`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/217-ivf-page-tuple-access-view`
- timestamp: `2026-05-21T10:59:38Z`
- storage format / rerank mode: not applicable; code-only unsafe structural cleanup slice
- isolated one-index-per-table or shared-table surface: not applicable; no benchmark or corpus run

## Artifacts

### `rustfmt-check.log`

- command: `rustfmt --check src/am/ec_ivf/page.rs`
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

### `cargo-test-ec-ivf-page-pg18-pgtest-no-run.log`

- command: `cargo test --lib ec_ivf::page --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: existing Hadamard test-helper dead-code warnings; `Finished test profile`; unit-test executable built.

## Unsafe Counts

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2548`
- `git grep -n "unsafe" HEAD -- src | wc -l`: `2548`
- `git grep -n "unsafe" HEAD^ -- src/am/ec_ivf/page.rs | wc -l`: `48`
- `git grep -n "unsafe" HEAD -- src/am/ec_ivf/page.rs | wc -l`: `48`
- `git grep -n "unsafe fn" HEAD^ -- src/am/ec_ivf/page.rs | wc -l`: `18`
- `git grep -n "unsafe fn" HEAD -- src/am/ec_ivf/page.rs | wc -l`: `18`

