# Task 39 ec_spire/storage/relation_store.rs shadow-crate scaffold

## Summary

Lifts `am/ec_spire/storage/relation_store.rs` off **0.00%** coverage
by mounting it inside the `hardening/careful` shadow crate (sister
slice to packet 033 which did the same for `am/ec_spire/page.rs`).

New baseline: **3.98%** line / 4.65% function / 1.78% region.

The file is 1408 source lines; the two tests added here only
exercise the construction-time early-error paths. Raising further
needs a backing-page emulator for the insert/read code paths and is
a separate slice.

## Code under review

- Code scaffold commit: `9a818d3360bfd4887adad241e0af6f78924caa44`
- Baseline ratchet commit: pending (this packet only edits
  `fixtures/quality/coverage-baseline.tsv`).
- Changed files: `hardening/careful/src/pg_guards.rs`,
  `hardening/careful/src/lib.rs`,
  `hardening/careful/src/spire.rs`,
  `fixtures/quality/coverage-baseline.tsv`.

## Scaffold details

Added to `hardening/careful/src/pg_guards.rs`:

- `RelationData` gains `rd_id: Oid` (threaded through
  `open_relation` so the existing relation-open helpers populate
  it).
- New constants: `BLCKSZ`, `InvalidOid`, `InvalidBuffer`,
  `READ_STREAM_DEFAULT`.
- New `ReadStream` type and stubs: `PrefetchBuffer`,
  `read_stream_begin_relation`, `read_stream_next_buffer`
  (returns `InvalidBuffer` so callers terminate gracefully),
  `read_stream_end`.

Wires `crate::storage::relation_guard` in the careful crate's
`storage` shim so `relation_store.rs`'s `use` statements resolve.
Adds `use super::page` + `use crate::careful_pg_guards::pg_sys` to
the `careful_spire::storage` block so the included
`relation_store.rs` picks them up. Adds the `include!` for
`storage/relation_store.rs` alongside the other storage submodules.

## Tests

Two new tests in `careful_spire::storage::tests`:

- `relation_object_store_for_index_relation_rejects_null_and_invalid_oid`
  — exercises both early-error paths in `for_index_relation`:
  null pointer and `rd_id == InvalidOid` (the latter constructed
  with a stack-allocated `pg_sys::RelationData` so the unsafe
  pointer access lands on real memory).
- `relation_object_store_inserts_reject_epoch_zero` — pins the
  `epoch == 0` guard on `insert_routing_object`,
  `insert_delta_object`, `insert_top_graph_object`, and
  `insert_leaf_object_v2_from_rows`.

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`:
  **463 passed** (was 461 before this packet).
- `make coverage`: `relation_store.rs` 0.00 → **3.98** line
  coverage (`artifacts/coverage/summary.txt`).
- `scripts/check_coverage_delta.sh` with the single-path
  changed-files list — `coverage ok:
  am/ec_spire/storage/relation_store.rs actual=3.98
  baseline=3.98`. Artifact:
  `artifacts/coverage-delta-check.log`.
- `scripts/check_coverage_baseline_complete.sh` — `coverage
  baseline complete for 40 critical paths`. Artifact:
  `artifacts/coverage-baseline-check.log`.

## Notes / follow-ups

- The remaining 96% of `relation_store.rs` lines (insert / read /
  chain / scan paths) need a backing-page emulator inside the
  pg_sys mocks (real `PageInit`/`PageAddItemExtended` semantics
  with a per-buffer 8K backing buffer and `ItemId` table). That's
  a multi-hour slice.
- `coordinator/diagnostics.rs` and `ec_diskann/routine.rs`
  remain at 0%. `diagnostics.rs` is `include!`'d into `mod.rs`
  rather than being a standalone module, so adding it to the
  careful crate needs a different surgery (its pgrx-touching
  half versus its pure-helper half).
