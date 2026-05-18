# Artifact Manifest: SPIRE Typed Tuple NULL And Array

- Packet: `30916-spire-typed-tuple-null-array`
- Head SHA: `9b0cb4f4971b834f3cda4da251ddceb1da5f0ccf`
- Timestamp: `2026-05-12T11:15:38-07:00`
- Isolated one-index-per-table or shared-table surface: local PG18 typed tuple
  endpoint fixtures using one table/index per test.

## Artifacts

### `git-diff-check.log`

- Head SHA: `9b0cb4f4971b834f3cda4da251ddceb1da5f0ccf`
- Lane / fixture / storage format / rerank mode: static diff check; not
  storage-format or rerank specific.
- Command: `git diff --check HEAD^ HEAD`
- Key result lines: command exited 0 with no whitespace errors.

### `cargo-fmt-check.log`

- Head SHA: `9b0cb4f4971b834f3cda4da251ddceb1da5f0ccf`
- Lane / fixture / storage format / rerank mode: formatting check; not
  storage-format or rerank specific.
- Command: `cargo fmt --check`
- Key result lines: command exited 0. Rustfmt printed the repository's existing
  stable-toolchain warnings for unstable import-grouping options.

### `cargo-pgrx-test-typed-null-array.log`

- Head SHA: `9b0cb4f4971b834f3cda4da251ddceb1da5f0ccf`
- Lane / fixture / storage format / rerank mode: PG18 typed tuple endpoint
  NULL and `text[]` array fixture; default SPIRE storage; rerank mode not
  applicable.
- Command: `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_null_array_sql`
- Key result lines:
  - `test tests::pg_test_ec_spire_typed_tuple_payload_null_array_sql ... ok`
  - `test result: ok. 1 passed; 0 failed`

### `cargo-pgrx-test-typed-tuple-payload.log`

- Head SHA: `9b0cb4f4971b834f3cda4da251ddceb1da5f0ccf`
- Lane / fixture / storage format / rerank mode: PG18 typed tuple endpoint
  scalar plus NULL/array fixtures; default SPIRE storage; rerank mode not
  applicable.
- Command: `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload`
- Key result lines:
  - `test tests::pg_test_ec_spire_typed_tuple_payload_null_array_sql ... ok`
  - `test tests::pg_test_ec_spire_typed_tuple_payload_scalar_parity_sql ... ok`
  - `test result: ok. 2 passed; 0 failed`
