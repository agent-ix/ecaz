# Task 41 FFI resource closeout and live smoke

## Summary

This packet requests review for a Task 41 invariant #1/#3 closeout slice.

Code commits:

- `ab9dc31524956e782efe6f573b2e023b9b61f8b6` closes the SPI guard gap and
  macOS RLIMIT smoke skip.
- `0de684c46607f137c70edf14a1bfdcace313dc09` removes a raw unused tuple-slot
  allocator and adds slot API lint coverage.

Changes:

- Added `src/storage/spi_guard.rs`, a `SpiTupleTableGuard` RAII wrapper that
  drops owned `pg_sys::SPITupleTable` pointers with `SPI_freetuptable`.
- Registered `storage::spi_guard` so the existing `ffi-lint` allow-list now
  points at a real wrapper module.
- Extended `scripts/ffi_lint.py --self-test` with a negative raw
  `SPI_freetuptable` fixture and an allowed wrapper-module fixture.
- Removed an unused raw `MakeSingleTupleTableSlot` helper from
  `src/am/ec_hnsw/source.rs`.
- Added a `scripts/ffi_lint.py` rule and self-test fixture that confine tuple
  slot allocation/release APIs to `src/storage/slot_guard.rs`.
- Made the `ecaz dev fault smoke` RLIMIT_AS OOM probe Linux-only and emit a
  structured skip on macOS, while keeping the Linux probe behavior intact.

## Safety Effect

Invariant #1 remains complete from packets 121-126: the FFI inventory reports
zero unguarded C ABI callbacks, `make ffi-audit` and the Dylint
`ecaz_panic_across_ffi` gate are clean, and their negative fixtures pass.

Invariant #3 now has all named Task 41 resource families represented by wrapper
modules and enforced by `make ffi-lint`: buffer pins/locks, LWLocks, snapshots,
relations, tuple slots, and SPI tuptables. The current source sweep has no raw
`SPI_freetuptable` or tuple slot allocation/release callsites outside wrapper
modules, and no direct `pg_sys::SPI_*` acquisition sites outside
`src/storage/spi_guard.rs`.

The live smoke here is PG18 HNSW evidence, not exhaustive all-AM runtime
coverage. It exercises palloc failure, backend crash recovery, cancel, timeout,
lock-timeout, temp accounting, WAL accounting, and buffercache pin postconditions
against an isolated database. Exhaustive source coverage is provided by the
static gates.

## Review Focus

- Check the `SpiTupleTableGuard` ownership contract and drop behavior.
- Check that the SPI and tuple-slot self-test fixtures close the
  reviewer-identified #3 gaps.
- Check that the non-Linux RLIMIT_AS skip is acceptable for macOS live smoke and
  does not weaken Linux coverage.
- Confirm packet-local artifacts support the #1/#3 status rollup without
  overlapping Agent IX's invariant #2 work.

## Validation

See `artifacts/manifest.md` for commands and key result lines.
