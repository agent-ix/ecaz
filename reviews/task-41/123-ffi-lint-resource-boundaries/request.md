# Task 41 FFI lint resource boundaries

## Summary

This packet requests review for a Task 41 invariant #1/#3 enforcement slice.
It adds the first `make ffi-lint` lane and wires it into `ci-quick`.

Code commit: `be5511caaea65a4093c8cc2dcabae451ced8f774`

Changes:

- Added `scripts/ffi_lint.py`, a repo-local static lint that fails when raw
  PostgreSQL resource APIs escape their guard modules.
- Added `python3 scripts/ffi_audit.py --self-test`, including an intentionally
  unguarded fixture string to prove the FFI audit catches the failure class.
- Added `make ffi-lint`, depending on `make ffi-audit`, then running the
  self-test and resource-boundary lint.
- Wired `ffi-lint` into `ci-quick`.
- Replaced a remaining raw `BufferGetBlockNumber(buffer.buffer())` call in
  `src/am/ec_spire/page.rs` with `LockedBufferGuard::block_number()`.
- Updated the unsafe baseline from `3687` to `3686`.

## Safety Effect

`make ffi-lint` now enforces two Task 41 boundaries:

- invariant #1: the FFI audit detector catches a deliberately unguarded direct
  C ABI fixture.
- invariant #3: raw buffer, LWLock, snapshot, relation, and SPI tuptable release
  APIs are confined to their storage guard modules; read-stream buffers must be
  adopted locally by `PinnedBufferGuard` or `LockedBufferGuard`.

The PR-tier `ci-quick` aggregate now includes `ffi-lint`, which also runs
`ffi-audit` through the target dependency.

## Review Focus

- Check the resource-boundary allow-list in `scripts/ffi_lint.py`.
- Check the `read_stream_next_buffer` local-adoption heuristic for the current
  read-stream call shapes.
- Check whether `ci-quick: ... ffi-lint` is the right governance hook for this
  low-cost Task 41 gate.

## Validation

See `artifacts/manifest.md` for the command log and key result lines.
