# Artifact Manifest

Task bucket: `reviews/task-41/`

Packet: `reviews/task-41/120-hnsw-concurrent-dsm-lock-guard/`

Head SHA: `ad8f6f0e45740a13da34b4cad55ebab8f146a328`

Timestamp: `2026-05-18T03:03:13Z`

Lane: Task 41 invariant #3, HNSW concurrent DSM graph lock release

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
wrote scripts/unsafe_comment_baseline.txt with 3691 entries
```

### `git diff --check`

Result: passed.

### `bash scripts/check_unsafe_comments.sh`

Result: passed.

### `bash scripts/unsafe_baseline_report.sh`

Result: passed.

Key lines:

```text
entries: 3691
files: 106
204 src/am/ec_hnsw/build_parallel.rs
```

Prior packet 119 baseline was `3698` entries and `211` entries for
`src/am/ec_hnsw/build_parallel.rs`.

### `rg -n "locks\\.(shared|exclusive)|locks\\.release|acquire_shared|acquire_exclusive|LWLockAcquire|LWLockRelease" src/am/ec_hnsw/build_parallel.rs`

Result: passed.

Key result lines:

```text
442:        unsafe { (self.acquire_shared)(lock) };
450:        unsafe { (self.acquire_exclusive)(lock) };
1324:    let _lock_guard = unsafe { locks.exclusive(lock) };
1350:    let _lock_guard = unsafe { locks.exclusive(lock) };
1524:        let _source_lock_guard = unsafe { locks.shared(source_lock) };
1620:        let _target_lock_guard = unsafe { locks.exclusive(target_lock) };
1804:    unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_SHARED) };
1808:    unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_EXCLUSIVE) };
1812:    unsafe { pg_sys::LWLockRelease(lock) };
```

There were no `locks.release` call-site matches.

### `make fmt-check`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `cargo check --all-targets --no-default-features --features pg18,bench`

Result: passed.

Known warnings:

- PG18 C headers emit unused-parameter warnings.
- Existing unused re-export warning in `src/am/mod.rs`.
