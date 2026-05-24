# Task 50 Packet 351 Artifact Manifest

- head SHA: `cb50c8eea2ad102f067f148a7e58d3bd4c961f32`
- parent SHA: `c388a821a8ab7ec2f320c39522eeda82828304d0`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/351-relation-scalar-nonnull-helpers/`
- timestamp: `2026-05-22T00:05:43Z`
- lane: Task 50 unsafe burndown, relation scalar handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF admin diagnostics, DiskANN graph diagnostics, HNSW scan debug, storage relation scalar readers

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
  - result: direct unsafe count is `1295`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `relation-scalar-helper-scan.log`
  - command: `rg -n 'unsafe \{ crate::storage::relation::main_fork_block_count|unsafe \{ crate::storage::relation::relation_reltuples|main_fork_block_count_handle|relation_reltuples_handle|type RelationHandle' src/am/ec_ivf/admin.rs src/am/ec_diskann/diagnostics.rs src/am/ec_hnsw/scan_debug.rs src/storage/relation.rs`
  - result: updated callers use the checked handle helpers; no targeted caller-side unsafe wrappers remain in those files
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/351-relation-scalar-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/351-relation-scalar-nonnull-helpers src`
  - result: wrote `1295` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/351-relation-scalar-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1295 current unsafe rows`
