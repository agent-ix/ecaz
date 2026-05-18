# Artifact Manifest

Task bucket: `reviews/task-41/`

Packet: `reviews/task-41/123-ffi-lint-resource-boundaries/`

Head SHA: `be5511caaea65a4093c8cc2dcabae451ced8f774`

Timestamp: `2026-05-18T04:33:49Z`

Lane: Task 41 invariant #1/#3 static lint enforcement

Fixture / storage format / rerank mode: not applicable; static lint lane

Index isolation: not applicable; no benchmark run

## Commands

### `python3 scripts/ffi_audit.py --self-test`

Result: passed.

Key line:

```text
ffi audit self-test passed
```

### `python3 scripts/ffi_lint.py --check`

Result: passed.

Key line:

```text
ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules
```

### `make ffi-lint`

Result: passed.

Key lines:

```text
python3 scripts/ffi_audit.py --check
ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints
python3 scripts/ffi_audit.py --self-test
ffi audit self-test passed
python3 scripts/ffi_lint.py --check
ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules
```

### `cargo fmt`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `bash scripts/check_unsafe_comments.sh --update-baseline`

Result: passed.

Key line:

```text
wrote scripts/unsafe_comment_baseline.txt with 3686 entries
```

### `git diff --check`

Result: passed.

### `bash scripts/check_unsafe_comments.sh`

Result: passed.

### `bash scripts/unsafe_baseline_report.sh`

Result: passed.

Key lines:

```text
entries: 3686
files: 106
61 src/am/ec_spire/page.rs
```

### `make ffi-audit`

Result: passed.

Key lines:

```text
python3 scripts/ffi_audit.py --check
ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints
```

### `rg -n "BufferGetBlockNumber|ReadBufferExtended|LockBuffer\\(|UnlockReleaseBuffer|ReleaseBuffer\\(|LWLockAcquire|LWLockRelease|RegisterSnapshot|UnregisterSnapshot|index_open\\(|index_close\\(|SPI_freetuptable" src -g '*.rs'`

Result: passed.

Key result: raw resource APIs matched only storage guard modules.

### `make fmt-check`

Result: passed.

Known warnings: stable rustfmt reports `imports_granularity` and
`group_imports` are nightly-only.

### `cargo check --all-targets --no-default-features --features pg18,bench`

Result: passed.

Known warnings:

- PG18 C headers emit unused-parameter warnings.
- Existing unused re-export warning in `src/am/mod.rs`.
