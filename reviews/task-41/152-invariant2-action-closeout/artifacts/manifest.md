# Artifact Manifest

- Head SHA: `52a6e1a5e68ff9ac7357a76ae1d829bdc375a7bf`
- Task bucket: `reviews/task-41/`
- Packet: `reviews/task-41/152-invariant2-action-closeout/`
- Lane: Task 41 invariant 2 lifetime/scope closeout
- Fixture: static Rust source review and PG18 compile checks
- Storage format: repository source
- Rerank mode: not applicable
- Timestamp: `2026-05-18T05:13:25Z`
- Surface: shared task-41 branch; no database storage surface used

## Artifacts

### `code-commit-stat.log`

- Command: `git show --stat --oneline --no-renames 52a6e1a5`
- Result: records the reviewed code commit touching `src/am/ec_hnsw/shared.rs` and `src/am/ec_hnsw/vacuum.rs`.

### `git-diff-check-code-commit.log`

- Command: `git diff --check 52a6e1a5^ 52a6e1a5`
- Result: passed with no whitespace errors.

### `cargo-fmt-check.log`

- Command: `script -q -c 'cargo fmt --all --check' reviews/task-41/152-invariant2-action-closeout/artifacts/cargo-fmt-check.log`
- Result: passed. The log contains stable rustfmt warnings for nightly-only config options.

### `cargo-check-all-targets-pg18.log`

- Command: `script -q -c 'cargo check --all-targets --no-default-features --features pg18' reviews/task-41/152-invariant2-action-closeout/artifacts/cargo-check-all-targets-pg18.log`
- Result: passed. The log records pre-existing warnings in `src/am/mod.rs` and `src/quant/hadamard.rs`.

### `cargo-clippy-all-targets-pg18.log`

- Command: `script -q -c 'cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings' reviews/task-41/152-invariant2-action-closeout/artifacts/cargo-clippy-all-targets-pg18.log`
- Result: failed with exit code 101 on broad existing lint debt, including unused reexports, existing `let_and_return`, `type_complexity`, `too_many_arguments`, `useless_conversion`, and test lint failures. This is recorded as an attempted acceptance check, not as Task 41 invariant 2 scope.

### `final-memory-lifetime-inventory.log`

- Command: `rg -n 'from_raw_parts|from_raw_parts_mut|varlena_to_byte_slice|tts_values.add|CStr::from_ptr|for<.*FnOnce' src`
- Result: final source inventory used to spot-check remaining raw byte and datum borrow surfaces after the cleanup slices.

### `git-status.log`

- Command: `script -q -c 'git status --short --branch' reviews/task-41/152-invariant2-action-closeout/artifacts/git-status.log`
- Result: branch status snapshot captured before packet commit.
