# Task 41 FFI audit inventory

## Summary

This packet requests review for a Task 41 invariant #1 slice: adding the
auditable FFI callback inventory and a `make ffi-audit` lane.

Code commit: `60d20e72c21d8877cf1aa0be6399948a7ba02264`

Changes:

- Added `scripts/ffi_audit.py`, which inventories direct C ABI functions and
  pgrx-managed SQL entrypoints under `src/`.
- Added `make ffi-audit`, wired to `python3 scripts/ffi_audit.py --check`.
- Added generated `docs/ffi-inventory.md`.
- Wrapped the remaining real callback bodies found by the audit with
  `pgrx::pgrx_extern_c_guard`:
  - PG17/PG18 `ec_amestimateparallelscan`
  - SPIRE DML frontdoor relcache callback
  - HNSW, IVF, SPIRE, and DiskANN debug vacuum dead-TID callbacks
- Left metadata-only `pg_finfo_*` symbols and the local test panic stub as
  documented exceptions in the inventory.

## Safety Effect

`make ffi-audit` now fails when a direct C ABI function lacks one of:

- `#[pg_guard]`
- `pgrx::pgrx_extern_c_guard`
- `std::panic::catch_unwind`
- a documented metadata/test exception

The generated inventory currently reports:

```text
Direct C ABI functions: 101
Guarded direct C ABI functions: 95
Documented direct C ABI exceptions: 6
Unguarded direct C ABI functions: 0
pgrx-managed SQL entrypoints: 288
```

## Review Focus

- Check that the scanner's direct-C-ABI coverage is a reasonable first
  enforcement lane for Task 41 invariant #1.
- Check that the six documented exceptions are correctly limited to
  metadata-only `pg_finfo_*` symbols plus the local unit-test panic stub.
- Check that wrapping `ec_amestimateparallelscan` and the debug callback
  helpers does not alter return semantics.

## Validation

See `artifacts/manifest.md` for the command log and key result lines.
