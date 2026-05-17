# 30764 - SPIRE Standalone pgrx Loader Stubs

## Summary

This packet reviews commit `afd97885ade67565c6a25b2f3a8a0aa0407e78d1`
(`Fix standalone PG18 test loader stubs`).

The slice responds to reviewer feedback on `30761` P3. Direct
`cargo test ... --features pg18` had compiled successfully but failed before
running focused pure Rust tests because the pgrx test binary still had
unresolved PostgreSQL backend symbols such as `SPI_finish`.

The standalone test stub library now exports the additional pgrx/Postgres
symbols needed for direct PG18 pure Rust tests to load. The safe helper symbols
are inert stubs: memory-context globals, minimal memory-context allocation,
transaction id helpers, binary coercion, type formatting, and SPI globals.
SPI execution/accessor functions still panic with an explicit "unavailable
outside a PostgreSQL backend" message, so direct tests cannot silently fake
database execution. Backend-facing `#[pg_test]` coverage still belongs in the
`cargo pgrx test` lane.

## Key Files

- `csrc/standalone_pg_backend_stubs.c`

## Validation

- `git diff --check -- csrc/standalone_pg_backend_stubs.c`
- `cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18`
- `cargo test row_materialization_contract --no-default-features --features pg18`
- `cargo check --no-default-features --features pg18`
- `cargo fmt --check`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `nm -u target/debug/deps/ecaz-320dfae4bc07716b | rg '<previously missing symbols>'`
  returned no matches after the focused test build.

No PostgreSQL server or distributed fixture was started for this packet.

## Review Focus

- Are the newly exported symbols narrow enough for the standalone pure Rust
  test lane?
- Is it correct for SPI execution/accessor stubs to panic instead of returning
  fake data?
- Does this leave the separation clear between direct `cargo test` for pure
  Rust behavior and `cargo pgrx test` for backend behavior?
