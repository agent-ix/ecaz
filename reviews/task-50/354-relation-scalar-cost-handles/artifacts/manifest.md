# Task 50 Packet 354 Artifact Manifest

- head SHA: `8c4c13bfeddbc4d11b230009349a721ff21da6b1`
- parent SHA: `ed93c4718b6963c1548faf12f303b9742c30ec7a`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/354-relation-scalar-cost-handles/`
- timestamp: `2026-05-22T00:20:31Z`
- lane: Task 50 unsafe burndown, relation scalar handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: DiskANN insert, HNSW vacuum/shared, IVF cost, SPIRE cost relation scalar reads

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
  - result: direct unsafe count is `1265`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `relation-scalar-cost-handle-scan.log`
  - command: `rg -n 'unsafe \{ crate::storage::relation::(main_fork_block_count|relation_reltuples)|main_fork_block_count_handle|relation_reltuples_handle' src/am/ec_diskann/insert.rs src/am/ec_hnsw/vacuum.rs src/am/ec_hnsw/shared.rs src/am/ec_ivf/cost.rs src/am/ec_spire/cost/mod.rs`
  - result: updated files use checked handle helpers; no targeted caller-side raw relation scalar wrappers remain in these files
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/354-relation-scalar-cost-handles/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/354-relation-scalar-cost-handles src`
  - result: wrote `1265` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/354-relation-scalar-cost-handles/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1265 current unsafe rows`
