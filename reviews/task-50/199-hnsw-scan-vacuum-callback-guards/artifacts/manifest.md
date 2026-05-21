# Task 50 Packet 199 Artifact Manifest

- Task bucket: `reviews/task-50/199-hnsw-scan-vacuum-callback-guards`
- Head SHA: `495b53f0ab31ecde828f0551d11ab573a958c0bd`
- Timestamp: `2026-05-21T08:53:28Z`
- Lane: unsafe burndown, HNSW scan/vacuum callback guard consolidation
- Fixture: source build checks only
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no benchmark or table/index fixture used

## Unsafe Ledger

- Touched files combined:
  - `src/am/ec_hnsw/scan.rs`
  - `src/am/ec_hnsw/vacuum.rs`
  - `unsafe` matches `309 -> 303`
- `src/`: `unsafe` matches `2635 -> 2629`

## Artifacts

### `rustfmt-hnsw-scan-vacuum.log`

- Command: `script -q -c "rustfmt --check src/am/ec_hnsw/scan.rs src/am/ec_hnsw/vacuum.rs" reviews/task-50/199-hnsw-scan-vacuum-callback-guards/artifacts/rustfmt-hnsw-scan-vacuum.log`
- Result: pass
- Key lines: stable rustfmt reported existing warnings for unstable `imports_granularity` and `group_imports` configuration, then completed successfully.

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/199-hnsw-scan-vacuum-callback-guards/artifacts/cargo-check-pg18-bench.log`
- Result: pass
- Key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
- Warnings: existing unused SPIRE imports in `src/am/mod.rs`.

### `cargo-check-pg18-pg-test.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/199-hnsw-scan-vacuum-callback-guards/artifacts/cargo-check-pg18-pg-test.log`
- Result: pass
- Key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.18s`
- Warnings: existing unused Hadamard test helper functions.

### `cargo-test-hnsw-no-run.log`

- Command: `script -q -c "cargo test --lib am::ec_hnsw --no-default-features --features pg18,pg_test --no-run" reviews/task-50/199-hnsw-scan-vacuum-callback-guards/artifacts/cargo-test-hnsw-no-run.log`
- Result: pass
- Key lines: `Executable unittests src/lib.rs (target/debug/deps/ecaz-c47ec6114f4ad275)`
- Warnings: existing unused Hadamard test helper functions.

### `git-diff-check.log`

- Command: `script -q -c "git diff --check HEAD" reviews/task-50/199-hnsw-scan-vacuum-callback-guards/artifacts/git-diff-check.log`
- Result: pass
- Key lines: no whitespace errors.
