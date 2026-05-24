# Artifact Manifest: Quantized Product Checked SIMD Dispatch

- head SHA: `5a04edc779dbc296564a7b94619d502be3cf026a`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/216-quant-prod-checked-simd-dispatch`
- timestamp: `2026-05-21T10:52:38Z`
- storage format / rerank mode: not applicable; code-only unsafe burn-down slice
- isolated one-index-per-table or shared-table surface: not applicable; no benchmark or corpus run

## Artifacts

### `rustfmt-check.log`

- command: `rustfmt --check src/quant/prod.rs`
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

### `cargo-test-quant-prod-pg18-bench-no-run.log`

- command: `cargo test --lib quant::prod --no-default-features --features pg18,bench --no-run`
- result: passed
- key lines: `Finished test profile`; unit-test executable built.

## Unsafe Counts

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2552`
- `git grep -n "unsafe" HEAD -- src | wc -l`: `2548`
- `git grep -n "unsafe" HEAD^ -- src/quant/prod.rs | wc -l`: `17`
- `git grep -n "unsafe" HEAD -- src/quant/prod.rs | wc -l`: `13`
- `git grep -n "unsafe fn" HEAD^ -- src/quant/prod.rs | wc -l`: `5`
- `git grep -n "unsafe fn" HEAD -- src/quant/prod.rs | wc -l`: `5`

