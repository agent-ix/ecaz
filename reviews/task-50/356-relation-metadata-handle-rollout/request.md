# Task 50 Review Request: Relation Metadata Handle Rollout

## Summary

This slice continues P2 PostgreSQL handle-view cleanup by moving relation metadata reads behind `RelationHandle` APIs and deleting now-unused raw relation wrappers.

Code commit: `39e8de240b32d19ef54309c3c96fb483a62497ad`

Changes:

- Added handle-only APIs for relation name, kind, AM OID, tablespace, namespace/owner/persistence, tuple descriptor copy, raw tuple descriptor copy, and reloptions.
- Removed unused raw-pointer wrappers for those metadata reads.
- Rolled the handle APIs through SPIRE, IVF/RaBitQ, DiskANN, HNSW, root index validation, and relation guard call sites.
- Added a `pg_test`/test-gated `IndexRelationGuard::handle()` helper for debug-only relation metadata use.

Unsafe count:

- Before: `1246`
- After: `1230`
- Delta: `-16`

Targeted scan result:

- No remaining calls to `crate::storage::relation::relation_{name,kind,am_oid,namespace_owner_persistence,tuple_desc_copy,raw_tuple_desc_copy,options,tablespace}(...)` under `src`.

## Validation

Artifacts are under `reviews/task-50/356-relation-metadata-handle-rollout/artifacts/`.

- `cargo-check-pg18-bench-final.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check-final.log`: `git diff --check` passed.
- `unsafe-count-final.log`: `1230`.
- `raw-boundary-guard-final.log`: no matches.
- `relation-metadata-raw-call-scan-final.log`: no matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check-final.log`: ledger regenerated and covers all `1230` current unsafe rows.

The initial cargo run passed but exposed a new dead-code warning for `IndexRelationGuard::handle()` in non-test builds. The helper was gated to `test`/`pg_test`, and the final cargo run returned to the known single pre-existing warning.
