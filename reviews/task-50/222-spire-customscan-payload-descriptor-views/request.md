# Review Request: SPIRE CustomScan Payload Descriptor Views

## Summary

Commit `24ef4e563be900ce2269158b7513065ca3ab1889` threads the existing typed `TupleDescView<'_>` through SPIRE CustomScan tuple-payload input metadata construction.

The change removes the safe `custom_scan_payload_attr_io(pg_sys::TupleDesc)` raw descriptor boundary. Raw descriptor validation now happens at the existing executor/slot boundaries:

- `custom_scan_tuple_payload_state_from_plan` builds a `TupleDescView<'_>` inside its existing unsafe executor callback boundary before calling `custom_scan_payload_attr_io`.
- `TupleSlotWriter` now exposes `tuple_desc_view()` instead of leaking the raw `TupleDesc` pointer back to SPIRE tuple-payload code.
- `custom_scan_payload_attr_io` consumes `&TupleDescView<'_>` and keeps only the PostgreSQL type-cache/fmgr initialization unsafe block that genuinely remains.

This is intentionally a typed-view cleanup, not a safe wrapper around a caller-supplied raw PG pointer.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2536 -> 2535`
- `src/am/ec_spire/custom_scan/dml.rs`: removed the raw tuple descriptor conversion from `custom_scan_payload_attr_io`.

## Validation

See `artifacts/manifest.md`.

- `rustfmt --check src/am/common/heap_slot.rs src/am/ec_spire/custom_scan/dml.rs src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/custom_scan/tuple_payload.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`

Known warnings only:

- stable-channel rustfmt import grouping warnings
- `src/am/mod.rs` unused SPIRE re-export warning
- Hadamard test-helper dead-code warnings
