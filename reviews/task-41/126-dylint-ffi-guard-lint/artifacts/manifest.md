# Artifact Manifest

Task bucket: `reviews/task-41/126-dylint-ffi-guard-lint`

Head SHA: `402885505408c4f3a0f7e75c8fe8175793f95ee5`

Timestamp: `2026-05-18T05:05:45Z`

## Artifacts

### `make-ffi-lint.log`

- Command: `script -q reviews/task-41/126-dylint-ffi-guard-lint/artifacts/make-ffi-lint.log make ffi-lint`
- Lane: Task 41 FFI audit/lint/Dylint validation
- Fixture: workspace plus Dylint negative fixture
- Storage format: source workspace, no benchmark data
- Rerank mode: not applicable
- Surface: static audit/lint only, no table/index surface
- Result: passed

Key result lines:

- `ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints`
- `ffi audit self-test passed`
- `ffi lint self-test passed`
- `ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules`
- `dylint self-test passed: /Users/peter/dev/tqvector/crates/ecaz-lints/target/panic_across_ffi.self-test.log`
- `Finished dev profile`

Notes:

- Dylint ran through `scripts/run_dylint.sh`, which pins `nightly-2026-04-16-aarch64-apple-darwin`, sets a repo-local Dylint driver path, and denies `ecaz_panic_across_ffi`.
- The log includes existing non-fatal PostgreSQL header warnings and an existing Rust unused-import warning.

### `dylint-self-test.log`

- Command: copied from `crates/ecaz-lints/target/panic_across_ffi.self-test.log` after `make ffi-lint`
- Lane: Dylint negative fixture
- Fixture: `crates/ecaz-lints/fixtures/panic_across_ffi`
- Storage format: source-only fixture
- Rerank mode: not applicable
- Surface: static lint fixture only
- Result: expected failure was observed and accepted by `scripts/run_dylint_self_test.sh`

Key result lines:

- `error: direct C ABI function body needs #[pg_guard], pgrx::pgrx_extern_c_guard, or catch_unwind`
- `src/lib.rs:3:1`
- `requested on the command line with -D ecaz-panic-across-ffi`

The self-test script also verifies that the guarded fixture functions are not reported.
