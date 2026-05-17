# Review Request: Task 41 HNSW parallel build worker relation guards

## Summary

Task 41 production follow-up for HNSW parallel heap build workers in
`src/am/ec_hnsw/build_parallel.rs`.

This slice replaces the worker's raw `table_open` / `index_open` handles and
manual close block with `HeapRelationGuard` and `IndexRelationGuard`. The worker
still passes raw `Relation` pointers to PostgreSQL callbacks, but the relation
lifetime is now owned by guards across worker scan setup, tuple production,
accounting, and shutdown.

Code commit: `95bb113b`

## Safety Effect

- Moves HNSW parallel worker heap relation close into `HeapRelationGuard::Drop`.
- Moves HNSW parallel worker index relation close into `IndexRelationGuard::Drop`.
- Hardens worker error paths so relation cleanup is not dependent on reaching
  the final manual close block.
- Keeps the unsafe baseline at `4097`; the remaining worker unsafe sites are
  shared-memory pointer/OID reads and callback calls rather than manual relation
  ownership.

## Review Focus

- Confirm guard lifetimes cover all uses of `heap_relation` and `index_relation`
  through `table_index_build_scan` and worker accounting.
- Confirm explicit `drop(index_relation_guard)` then `drop(heap_relation_guard)`
  preserves the previous close order.
- Confirm lock modes are unchanged for concurrent and non-concurrent workers.
- Confirm this does not affect leader-owned relation lifetimes.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
