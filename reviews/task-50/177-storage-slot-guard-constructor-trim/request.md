# Review Request: Storage Slot Guard Constructor Trim

## Scope

This packet reviews commit `43e090b87295878e0a17d3d648cedac793dc63ed` (`Trim raw tuple slot guard constructor`).

The slice closes a small piece of the storage guard soundness follow-up:

- removed the unused raw `TupleTableSlotGuard::create(pg_sys::Relation)` constructor;
- kept `create_for_heap_guard(&HeapRelationGuard)` as the only table-slot-create entry point;
- kept raw `single_for_heap(pg_sys::Relation)` as an explicit `unsafe fn` raw-boundary constructor for callback-owned relations;
- inlined the guarded constructors so they preserve the borrowed relation lifetime without routing through a raw constructor.

## Unsafe Burndown

Unsafe ledger movement:

- previous packet 176 ledger: `1849`
- packet 177 ledger: `1848`
- net reduction: `1`

High-signal file counts from `make unsafe-block-count`:

- `src/storage/slot_guard.rs`: `5 -> 4`

## Validation

Packet-local artifacts are under `reviews/task-50/177-storage-slot-guard-constructor-trim/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that removing the raw `create` constructor is the right boundary split: guard-borrowed creation remains safe and lifetime-bound, while raw relation construction remains available only for the single-tuple-slot callback cases that still require an `unsafe fn`.
