# Task 50 Review Request: StringInfo Receive Reader

## Summary

This slice moves PostgreSQL type receive buffer extraction behind the scoped
`StringInfoReader` API.

Changes:

- `src/storage/string_info.rs` adds `StringInfoReader::from_internal`, which
  extracts the live `StringInfoData` from a receive-function `Internal` once.
- `src/lib.rs` changes `recv_raw_f32_message` and `recv_tqvector_message` to
  accept a scoped `StringInfoReader` instead of raw `pg_sys::StringInfo`.
- `ecvector_recv` and `tqvector_recv` no longer build raw `StringInfo` pointers
  or call unsafe decode helpers directly.

The direct `unsafe { ... }` count moved from `1201` to `1199`.

Code commit: `9165f2e4d12acab0e4736364dcf944670fd06bd2`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- StringInfo receive scan confirms raw `pg_sys::StringInfo` handling is
  centralized in `src/storage/string_info.rs`; root receive functions use
  `StringInfoReader::from_internal`.
- Unsafe ledger check passed for `1199` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
