# Review Request: Index Heap Relation NonNull Helpers

Task: 50 unsafe burndown

Commit under review:

- `08ebb8ec` - `Use NonNull index heap relation helpers`

## Summary

This packet adds typed helpers for relation OID and index-to-heap OID reads, then rolls them into AM call sites.

- Adds `relation_oid_handle` and `index_heap_relation_oid_handle` over the existing `RelationHandle` alias.
- Keeps legacy raw-pointer helpers as checked wrappers.
- Updates HNSW build/scan, SPIRE scan relation resolution, DiskANN scan/vacuum relation resolution, and `IndexRelationGuard::heap_relation_oid` to pass checked handles.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1295` to `1286`.
- The updated call sites no longer wrap `index_heap_relation_oid` in caller-side unsafe blocks.
- The broadened raw boundary-signature guard has no hits.

See `artifacts/unsafe-count.log`, `artifacts/index-heap-relation-helper-scan.log`, and `artifacts/raw-boundary-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1286` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the `RelationHandle` helpers preserve the previous relation OID and index-to-heap OID semantics.
- Confirm the changed AM call sites reject null relation pointers before using the typed helpers.
