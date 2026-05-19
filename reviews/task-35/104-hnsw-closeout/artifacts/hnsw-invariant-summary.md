# HNSW Unsafe Boundary Summary

## Core Invariant Graph

- PostgreSQL AM callbacks enter through `pgrx_extern_c_guard` wrappers or callback-duration test helpers; raw PostgreSQL relation, slot, snapshot, and TID pointers are not retained beyond the callback scope.
- Index metadata is stored in block 0 and rewritten through `GenericXLogTxn` with an exclusive metadata buffer lock; readers borrow metadata page bytes only while the buffer guard pins the page.
- Data page tuple access follows the line-pointer chain: page pointer -> `pd_lower` line pointer count -> checked item id -> tuple offset/length bounds -> borrowed tuple byte slice.
- Graph storage decoding is driven by metadata-derived `GraphStorageDescriptor`; tuple tag and layout checks precede TurboQuant, hot/cold, and grouped hot tuple decoding.
- Scan state owns its candidate caches, frontier sets, result queues, source vector scratch, and heap rerank slots for the scan lifetime. Candidate tuple and source-vector borrows are scoped to the active buffer/slot guard.
- Insert/vacuum source scoring uses the same heap relation + snapshot + reusable slot chain; source vectors are copied or consumed before slots are cleared.
- Parallel build state is DSM-owned. Node indexes are bounds-checked before slice derivation, and PostgreSQL atomic fields are accessed under the established lock-or-atomic protocol.

## RAII And Resource Guards

- `LockedBufferGuard` owns pin and lock lifetimes for metadata/data page reads and writes.
- `GenericXLogTxn` scopes WAL page registrations and finishes after page bytes are initialized or rewritten.
- Relation guards such as `IndexRelationGuard` and heap relation guards keep test/debug relations open across direct AM helper calls.
- Tuple slot guards in callers own reusable heap slots and define when fetched row versions can be inspected and cleared.
- Read-stream state for PG18 prefetch is stack-owned and closed with `read_stream_end` after all buffers are consumed.

## Deferred Task 50 Candidates

- DSM atomic field wrapper: consolidate PostgreSQL atomic load/store/exchange calls behind a typed safe API with one constructor-level unsafe contract.
- AM callback guard helper: reduce repeated raw callback pointer comments by centralizing the pgrx guard and callback-duration pointer contract.
- Page tuple visitor wrapper: convert the repeated line-pointer bounds pattern into safe immutable/mutable tuple visitor APIs over a locked buffer guard.
- Heap source scorer helper: make the heap relation + snapshot + reusable slot lifecycle an owned safe object shared by insert, vacuum, and rerank paths.

## Residual Scope

No `src/am/ec_hnsw` production-source entries remain in the unsafe-comment baseline. HNSW-named test files under `src/tests/` remain for the Task 35 test-only sweep.
