# Task 50 Review Request: SPIRE Tuple Slot Writer Guard

## Summary

This packet adds a reusable `TupleSlotWriter` guard beside the existing tuple slot readers in `src/am/common/heap_slot.rs`, then moves SPIRE custom scan tuple payload storage onto that guard.

The SPIRE JSON and typed binary payload writers no longer reach directly into `TupleTableSlot` value/null arrays or tuple descriptor attributes. The new guard validates the slot and descriptor once, exposes safe attribute iteration metadata, centralizes null/datum writes, and owns the `ExecClearTuple` / `ExecStoreVirtualTuple` slot lifecycle calls.

This is a structural cleanup pass rather than only a local wrapper move: it removes direct unsafe from the SPIRE payload writers and leaves the remaining slot internals behind a named reusable boundary.

## Code Under Review

- Code commit: `97578040 Add SPIRE tuple slot writer guard`
- Files changed:
  - `src/am/common/heap_slot.rs`
  - `src/am/ec_spire/custom_scan/mod.rs`
  - `src/am/ec_spire/custom_scan/tuple_payload.rs`
  - `src/am/ec_spire/custom_scan/begin_exec.rs`

## Unsafe Ledger

- Touched files combined: `unsafe` matches `50 -> 46`
- `src/`: `unsafe` matches `2644 -> 2640`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/common/heap_slot.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_spire::custom_scan --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass

## Review Focus

- Confirm `TupleSlotWriter` preserves the previous `TupleTableSlot` write sequence: clear slot, populate value/null arrays, set `tts_nvalid`, then store virtual tuple.
- Confirm dropped/null descriptor attributes still produce NULL output values.
- Confirm typed payload attnum/name validation still happens before binary receive conversion.
- Confirm making `custom_scan_store_tuple_payload_json` / typed storage safe does not hide a caller-owned invariant without a named guard contract; the raw slot invariant should now be owned by `tuple_payload_writer`.
