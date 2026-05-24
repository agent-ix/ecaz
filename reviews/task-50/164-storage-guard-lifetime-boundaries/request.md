# Review Request: Storage Guard Lifetime Boundaries

## Summary

This checkpoint addresses the remaining confirmed finding from the Task 50 soundness audit: scan and tuple-slot guards had unsafe constructors but no lifetime marker tying the guard to the relation/snapshot dependency it was built from.

Code commit: `5c3ee5aeaabb3779fe0f4f0c1071680fd8d9172d`

## Scope

- Added lifetime parameters and `PhantomData` dependency markers to:
  - `IndexScanGuard<'heap, 'index, 'snap>`
  - `HeapScanGuard<'rel, 'snap>`
  - `TupleTableSlotGuard<'rel>`
- Kept the ordinary `IndexScanGuard::begin` constructor tied to borrowed `HeapRelationGuard`, `IndexRelationGuard`, and `ActiveSnapshotGuard`.
- Split out `IndexScanGuard::begin_from_raw` for the debug states that own the relation/snapshot guards in the same struct; those call sites now document the field-order/drop-order invariant explicitly.
- Added guard-borrowed tuple-slot constructors for heap relation guards:
  - `TupleTableSlotGuard::create_for_heap_guard`
  - `TupleTableSlotGuard::single_for_heap_guard`
- Migrated available guard-owned call sites to the borrowed constructors.
- Annotated long-lived callback-owned tuple slot fields as `TupleTableSlotGuard<'static>` where PostgreSQL callback state, not a Rust relation guard borrow, owns the underlying relation lifetime.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/storage/scan_guard.rs` | 5 | 6 |
| `src/storage/slot_guard.rs` | 3 | 5 |
| `src/am/ec_hnsw/scan_debug.rs` | 134 | 135 |
| `src/tests/custom_scan.rs` | 15 | 14 |
| Other touched files | unchanged | unchanged |

Current packet-local `src/` unsafe ledger: `1942` rows, checked.

This is a soundness-contract restoration slice, not a count-reduction slice. The added direct unsafe blocks are the explicit raw-constructor boundaries required to keep guard dependencies visible.

## Completion Audit Note

I audited the current state against `reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md` before this packet. Task 50 is not complete:

- `make unsafe-block-count` still reports direct unsafe across many `src/` files.
- The packet-local ledger still covers `1942` current `src/` unsafe rows.
- There is no final residual registry proving every remaining unsafe is irreducible with owner/invariant/validation.
- There is no final packet reporting separate `src`, hardening/crates, tests, and vendor disposition counts.

Therefore the closeout gate from packet 030 is not satisfied.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
