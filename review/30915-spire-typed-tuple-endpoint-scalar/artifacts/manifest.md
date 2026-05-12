# Artifact Manifest: SPIRE Typed Tuple Endpoint Scalar

- Packet: `30915-spire-typed-tuple-endpoint-scalar`
- Head SHA: `92e63f1bffce721006a202d36cc2161770be17f6`
- Timestamp: `2026-05-12T11:00:03-07:00`
- Isolated one-index-per-table or shared-table surface: local PG18 loopback
  fixtures using one table/index per test.

## Artifacts

### `git-diff-check.log`

- Head SHA: `92e63f1bffce721006a202d36cc2161770be17f6`
- Lane / fixture / storage format / rerank mode: static diff check; not
  storage-format or rerank specific.
- Command: `git diff --check HEAD^ HEAD`
- Key result lines: command exited 0 with no whitespace errors.

### `cargo-fmt-check.log`

- Head SHA: `92e63f1bffce721006a202d36cc2161770be17f6`
- Lane / fixture / storage format / rerank mode: formatting check; not
  storage-format or rerank specific.
- Command: `cargo fmt --check`
- Key result lines: command exited 0. Rustfmt printed the repository's existing
  stable-toolchain warnings for unstable import-grouping options.

### `cargo-pgrx-test-typed-scalar-parity.log`

- Head SHA: `92e63f1bffce721006a202d36cc2161770be17f6`
- Lane / fixture / storage format / rerank mode: PG18 typed tuple endpoint
  scalar JSON-parity fixture; default SPIRE storage; rerank mode not
  applicable.
- Command:
  `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_scalar_parity_sql`
- Key result lines:
  - `test tests::pg_test_ec_spire_typed_tuple_payload_scalar_parity_sql ... ok`
  - `test result: ok. 1 passed; 0 failed`

### `cargo-pgrx-test-json-tuple-payload-regression.log`

- Head SHA: `92e63f1bffce721006a202d36cc2161770be17f6`
- Lane / fixture / storage format / rerank mode: PG18 JSON tuple endpoint
  regression fixtures; default SPIRE storage; rerank mode not applicable.
- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_tuple_payload`
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_tuple_payload_missing_ctid_signal ... ok`
  - `test tests::pg_test_ec_spire_remote_search_tuple_payload_side_channel ... ok`
  - `test result: ok. 2 passed; 0 failed`
