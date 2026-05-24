# Task 50 Review Request: StringInfo Reader Boundaries

## Summary

This slice advances Task 50 storage boundary minimization for PostgreSQL
receive buffers.

Code commit: `5660ef507439d91d86f7ffd5fd0cd0fd0cfd96f6`

Changed `src/storage/string_info.rs` so each `StringInfoReader` operation keeps
its related PostgreSQL receive-buffer work in one explicit boundary block:

- `from_internal` now extracts the live `StringInfo` and constructs the raw
  reader in one block.
- `remaining_len` reads `len` and `cursor` in one block.
- `read_bytes` advances the PostgreSQL cursor and copies the returned byte
  range in one block.

The raw `StringInfo` constructor remains unsafe; this does not introduce a safe
raw PostgreSQL pointer API.

## Unsafe Counts

- `src/storage/string_info.rs`: `7 -> 4`
- `src/` total direct unsafe blocks: `1183 -> 1180`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P1/P6, FFI receive boundary and datum/vector payload decode support.
- Wave/tranche: Wave 4, storage guard residual minimization.
- Disposition: adjacent receive-buffer unsafe operations were consolidated
  behind the existing typed `StringInfoReader` contract.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/storage/string_info.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1180` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
