# Artifact Manifest

Task bucket: `reviews/task-41/`

Packet: `reviews/task-41/122-lwlock-storage-guard/`

Head SHA: `d4fbf0b0952cbb15f3faa92cab75633991c38d69`

Timestamp: `2026-05-18T04:27:05Z`

Lane: Task 41 invariant #3, shared LWLock RAII guard

Fixture / storage format / rerank mode: not applicable; structural code slice

Index isolation: not applicable; no benchmark run

## Commands

### `cargo fmt`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `bash scripts/check_unsafe_comments.sh --update-baseline`

Result: passed.

Key line:

```text
wrote scripts/unsafe_comment_baseline.txt with 3687 entries
```

### `python3 scripts/ffi_audit.py --write`

Result: passed.

Key line:

```text
wrote docs/ffi-inventory.md
```

### `git diff --check`

Result: passed.

### `bash scripts/check_unsafe_comments.sh`

Result: passed.

### `bash scripts/unsafe_baseline_report.sh`

Result: passed.

Key lines:

```text
entries: 3687
files: 106
201 src/am/ec_hnsw/build_parallel.rs
```

### `make ffi-audit`

Result: passed.

Key lines:

```text
python3 scripts/ffi_audit.py --check
ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints
```

### `rg -n "LWLockAcquire|LWLockRelease|LwLockGuard|EcHnswConcurrentDsmLockGuard|concurrent_dsm_lwlock" src/am/ec_hnsw/build_parallel.rs src/storage/lock_guard.rs src/storage/mod.rs`

Result: passed.

Key result lines:

```text
src/storage/lock_guard.rs:13:        unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_SHARED) };
src/storage/lock_guard.rs:19:        unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_EXCLUSIVE) };
src/storage/lock_guard.rs:57:    unsafe { pg_sys::LWLockRelease(lock) };
src/am/ec_hnsw/build_parallel.rs:435:            acquire_shared: LwLockGuard::acquire_shared,
src/am/ec_hnsw/build_parallel.rs:436:            acquire_exclusive: LwLockGuard::acquire_exclusive,
src/am/ec_hnsw/build_parallel.rs:3881:        unsafe { LwLockGuard::from_acquired_with_release(lock, test_lock_noop_release) }
```

There were no `EcHnswConcurrentDsmLockGuard` or `concurrent_dsm_lwlock` matches.

### `make fmt-check`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `cargo check --all-targets --no-default-features --features pg18,bench`

Result: passed.

Known warnings:

- PG18 C headers emit unused-parameter warnings.
- Existing unused re-export warning in `src/am/mod.rs`.
