# Task 50 Packet 198 Artifact Manifest

- Task bucket: `reviews/task-50/198-hnsw-build-insert-option-callback-guards`
- Head SHA: `3acd45a758caedea232fc0b1d2bdc86432d37436`
- Timestamp: `2026-05-21T08:47:30Z`
- Lane: unsafe burndown, HNSW callback guard consolidation
- Fixture: source build checks only
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no benchmark or table/index fixture used

## Unsafe Ledger

- Touched files combined:
  - `src/am/ec_hnsw/options.rs`
  - `src/am/ec_hnsw/insert.rs`
  - `src/am/ec_hnsw/build.rs`
  - `unsafe` matches `163 -> 158`
- `src/`: `unsafe` matches `2640 -> 2635`

## Artifacts

### `rustfmt-hnsw-callbacks.log`

- Command: `script -q -c "rustfmt --check src/am/ec_hnsw/options.rs src/am/ec_hnsw/insert.rs src/am/ec_hnsw/build.rs" reviews/task-50/198-hnsw-build-insert-option-callback-guards/artifacts/rustfmt-hnsw-callbacks.log`
- Result: pass
- Key lines: stable rustfmt reported existing warnings for unstable `imports_granularity` and `group_imports` configuration, then completed successfully.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/198-hnsw-build-insert-option-callback-guards/artifacts/cargo-check-pg18-bench.log`
- Result: pass
- Key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.17s`
- Warnings: existing unused SPIRE imports in `src/am/mod.rs`.

### `cargo-check-pg18-pg-test.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/198-hnsw-build-insert-option-callback-guards/artifacts/cargo-check-pg18-pg-test.log`
- Result: pass
- Key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`
- Warnings: existing unused Hadamard test helper functions.

### `cargo-test-hnsw-no-run.log`

- Command: `script -q -c "cargo test --lib am::ec_hnsw --no-default-features --features pg18,pg_test --no-run" reviews/task-50/198-hnsw-build-insert-option-callback-guards/artifacts/cargo-test-hnsw-no-run.log`
- Result: pass
- Key lines: `Executable unittests src/lib.rs (target/debug/deps/ecaz-c47ec6114f4ad275)`
- Warnings: existing unused Hadamard test helper functions.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check HEAD" reviews/task-50/198-hnsw-build-insert-option-callback-guards/artifacts/git-diff-check.log`
- Result: pass
- Key lines: no whitespace errors.
