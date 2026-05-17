# Review Request: Task 42 WAL Format Policy

## Summary

This checkpoint closes the Task 42 WAL version-tag item without inventing custom
WAL records that ECAZ does not currently emit.

Code commit: `c63e8e5adf2c0af0457223c7d0e893edc686ebcd` (`Document Task 42 WAL format policy`)

Changes:

- Documented that current ECAZ writes use PostgreSQL GenericXLog page
  images/deltas, not extension-owned WAL payload bodies.
- Added explicit constants in `src/storage/wal.rs`:
  - `ECAZ_CUSTOM_WAL_RECORDS_ENABLED = false`
  - `ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION = 1`
  - `ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET = 0`
- Added `validate_custom_wal_record_format_version` for future Task 37 custom
  redo/replay payloads.
- Added `tests/wal_policy.rs` to pin the policy and the reject path.
- Removed WAL as a remaining Task 42 gap in `docs/on-disk-format.md`; Task 37
  remains responsible for wiring the validator into any future custom replay
  path.

## Validation

Packet-local artifacts are under `artifacts/`.

- `cargo test --features bench --test wal_policy`: passed (`2 passed`).
- `cargo fmt --all -- --check`: passed with existing stable-toolchain warnings
  about unstable rustfmt options.

`cargo test --lib storage::wal` was not used as final validation because the
lib-test binary loads pgrx callback symbols and fails at process startup with
the existing unresolved `LockBuffer` symbol. The integration test keeps this
policy surface runnable without loading that lib-test binary.

## Reviewer Focus

- Does this accurately encode the current WAL reality: GenericXLog only, with
  page payload version tags carrying the replayed byte contract?
- Is the byte-0 custom WAL tag policy sufficient for Task 37 to consume when it
  adds extension-owned redo/replay records?
