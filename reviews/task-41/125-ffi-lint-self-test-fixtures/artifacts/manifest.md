# Task 41 FFI lint self-test fixture artifact manifest

- Head SHA: `d2a0c82e3fb2eda012388be51bbc24990746e56a`
- Task bucket: `reviews/task-41/`
- Packet path: `reviews/task-41/125-ffi-lint-self-test-fixtures/`
- Timestamp: `2026-05-18T04:42:22Z`
- Lane: Task 41 invariant #3 / FFI resource-boundary lint enforcement
- Fixture: built-in `scripts/ffi_lint.py --self-test` source fixtures
- Storage format: not applicable
- Rerank mode: not applicable
- Index/table isolation: not applicable

## Commands

### `python3 scripts/ffi_lint.py --self-test`

Purpose: verify the lint catches deliberately bad raw-resource fixtures and
does not flag wrapper-module or locally adopted read-stream fixtures.

Result:

```text
ffi lint self-test passed
```

### `make ffi-lint`

Purpose: run the full Task 41 FFI lint lane, including FFI inventory drift
check, audit self-test, new lint self-test, and production-code raw resource
scan.

Key output:

```text
python3 scripts/ffi_audit.py --check
ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints
python3 scripts/ffi_audit.py --self-test
ffi audit self-test passed
python3 scripts/ffi_lint.py --self-test
ffi lint self-test passed
python3 scripts/ffi_lint.py --check
ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules
```

### `git diff --check`

Purpose: whitespace validation for the code slice.

Result: passed.

