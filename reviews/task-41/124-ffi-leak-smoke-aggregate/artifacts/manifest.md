# Task 41 FFI leak smoke aggregate artifact manifest

- Head SHA: `8f2e6acdb466fa495de333d36d9ae7df143ab7c7`
- Task bucket: `reviews/task-41/`
- Packet path: `reviews/task-41/124-ffi-leak-smoke-aggregate/`
- Timestamp: `2026-05-18T04:36:39Z`
- Lane: Task 41 invariant #1 / FFI leak smoke aggregate
- Fixture: not applicable
- Storage format: not applicable
- Rerank mode: not applicable
- Index/table isolation: not applicable

## Commands

### `make -n ffi-leak-smoke`

Purpose: verify aggregate target wiring without executing live fault-smoke lanes.

Key output:

```text
cargo run -p ecaz-cli -- dev fault smoke --lane memory --dry-run
cargo run -p ecaz-cli -- dev fault smoke --lane cancel --dry-run
cargo run -p ecaz-cli -- dev fault smoke --lane timeout --dry-run
cargo run -p ecaz-cli -- dev fault smoke --lane lock-timeout --dry-run
cargo run -p ecaz-cli -- dev fault smoke --lane resource --dry-run
```

### `git diff --check`

Purpose: whitespace validation for the code slice.

Result: passed.

### `make ffi-lint`

Purpose: verify FFI audit inventory and resource-boundary lint lane still pass
after adding the aggregate target.

Result: passed.

### `rg -n "ffi-leak-smoke|fault-mem-smoke|fault-cancel-smoke|fault-timeout-smoke|fault-lock-smoke|fault-resource-smoke" Makefile`

Purpose: targeted confirmation of aggregate target membership.

Key output:

```text
348:.PHONY: ffi-leak-smoke
349:ffi-leak-smoke: fault-mem-smoke fault-cancel-smoke fault-timeout-smoke fault-lock-smoke fault-resource-smoke
```

## Notes

Live `make ffi-leak-smoke` was not run in this checkpoint; only dry-run target
expansion was validated.

