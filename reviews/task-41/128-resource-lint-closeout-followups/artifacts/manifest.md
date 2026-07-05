# Task 41 packet 128 artifact manifest

- Head SHA: `d78d0485a0b89527c7737cd032e44c46feb8ca45`
- Task bucket: `reviews/task-41/128-resource-lint-closeout-followups`
- Timestamp: `2026-05-18T15:07:45Z`
- Scope: Task 41 invariant #1/#3 deep-audit follow-ups only. Invariant #2 is
  intentionally untouched.

## Artifacts

### `cargo-check-ecaz-cli.log`

- Command:
  `script -q reviews/task-41/128-resource-lint-closeout-followups/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli`
- Purpose: compile-check the shared `RelationGuard` migration and lint/doc
  follow-up commit.
- Key result:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.17s`
- Notes:
  - Warnings are existing PG18 C-header warnings and existing unused imports in
    `src/am/mod.rs`.

### `make-ffi-lint.log`

- Command:
  `script -q reviews/task-41/128-resource-lint-closeout-followups/artifacts/make-ffi-lint.log make ffi-lint`
- Purpose: rerun the full Task 41 static gate after expanding resource lint
  coverage.
- Key result lines:
  - `ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints`
  - `ffi audit self-test passed`
  - `ffi lint self-test passed`
  - `ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules`
  - `dylint self-test passed`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.10s`

### `raw-resource-sweep.log`

- Command:
  `rg -n "pg_sys::(?:heap_beginscan|heap_endscan|table_open|table_close|relation_open|relation_close|PushActiveSnapshot|PopActiveSnapshot|index_beginscan|index_endscan|MakeSingleTupleTableSlot|table_slot_create|ExecDropSingleTupleTableSlot|SPI_freetuptable)\b" src -g '*.rs'`
- Purpose: packet-local source sweep for the APIs newly covered by
  `ffi_lint.py` plus the prior tuple/SPI families.
- Key result:
  - All listed callsites are under `src/storage/scan_guard.rs`,
    `src/storage/slot_guard.rs`, `src/storage/snapshot_guard.rs`,
    `src/storage/spi_guard.rs`, or `src/storage/relation_guard.rs`.
