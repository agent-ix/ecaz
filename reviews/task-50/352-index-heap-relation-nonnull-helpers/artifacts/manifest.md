# Task 50 Packet 352 Artifact Manifest

- head SHA: `08ebb8ec45e45393d12eec948be0f140ebfe929c`
- parent SHA: `bf5b21e960cc11caff2c81a8609a676dc723628f`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/352-index-heap-relation-nonnull-helpers/`
- timestamp: `2026-05-22T00:10:13Z`
- lane: Task 50 unsafe burndown, index heap relation handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: HNSW build/scan, SPIRE scan relation, DiskANN scan/vacuum, relation guard heap OID helper

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed
- `unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: direct unsafe count is `1286`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `index-heap-relation-helper-scan.log`
  - command: `rg -n 'unsafe \{ crate::storage::relation::index_heap_relation_oid|index_heap_relation_oid_handle|relation_oid_handle|type RelationHandle' src/storage/relation.rs src/storage/relation_guard.rs src/am/ec_hnsw/build.rs src/am/ec_hnsw/scan.rs src/am/ec_spire/scan/relation.rs src/am/ec_diskann/scan_state.rs src/am/ec_diskann/routine.rs`
  - result: updated call sites use checked handle helpers; no targeted caller-side `unsafe { index_heap_relation_oid(...) }` wrappers remain in these files
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/352-index-heap-relation-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/352-index-heap-relation-nonnull-helpers src`
  - result: wrote `1286` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/352-index-heap-relation-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1286 current unsafe rows`
