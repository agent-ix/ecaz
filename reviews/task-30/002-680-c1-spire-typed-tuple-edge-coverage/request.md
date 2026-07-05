# Review Request: SPIRE Typed Tuple Edge Coverage

- Code commit: `6b14168e` (`Cover SPIRE typed tuple edge cases`)
- Scope: Phase 12c.12 typed tuple transport coverage.
- File changed: `src/tests/remote_search/tuple_heap.rs`

## What Changed

- Added `test_ec_spire_typed_tuple_payload_null_array_wire_bytes_sql`.
  - Covers a real `NULL::text[]` projection separately from an empty `text[]`.
  - Pins `payload_nulls = ARRAY[false, true]` for the NULL array row.
  - Pins the NULL value bytes as `''::bytea` and asserts those bytes are not equal to `array_send(ARRAY[]::text[])`.
  - Pins the non-NULL empty array row as `payload_nulls = ARRAY[false, false]` with `payload_values[2] = array_send(ARRAY[]::text[])` and non-empty bytes.
- Added `test_ec_spire_typed_tuple_payload_composite_only_sql`.
  - Covers composite typed transport without the existing domain wrapper.
  - Pins metadata arrays, binary transport format, and `record_send(...)` payload bytes for the composite value.

## File-Size Discipline

`src/tests/remote_search/tuple_heap.rs` is now 887 lines, still well below the 2,500-line target. This keeps the new typed-transport coverage in the existing focused concern file instead of growing one of the oversized integration files.

## Validation

- `cargo fmt --check` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_typed_tuple_payload_null_array_wire_bytes_sql --no-run` passed.
- Runtime attempts are blocked in this local session before test execution by unresolved PostgreSQL backend symbols:
  - `cargo test --no-default-features --features pg18 test_ec_spire_typed_tuple_payload_null_array_wire_bytes_sql -- --nocapture` exits with missing `CacheRegisterRelcacheCallback`.
  - `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_null_array_wire_bytes_sql` exits with missing `pg_re_throw`.
  - The same failure happens under the prior `script -q -e -c "cargo test ..."` pattern.

## Review Focus

1. Confirm the visible typed-payload contract for NULL attributes is correctly represented as `payload_nulls[n] = true` plus empty value bytes, with the non-NULL empty array distinguished by `array_send(ARRAY[]::text[])`.
2. Confirm the composite-only fixture meaningfully separates pure composite transport from the existing domain+composite mixed fixture.
3. Decide whether reviewer wants a packet-local runtime artifact rerun from an environment where the pgrx test binary can load PostgreSQL backend symbols.
