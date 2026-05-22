# Task 50 Packet 353 Artifact Manifest

- head SHA: `9582e11af5de23f2c2bfc303ed41341719809b3a`
- parent SHA: `d3425fb01ad7ecb8dd3aa95ce794e1e11ee36653`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/353-relation-oid-nonnull-helpers/`
- timestamp: `2026-05-22T00:15:31Z`
- lane: Task 50 unsafe burndown, relation OID handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF insert lock OID, SPIRE insert/store/snapshot relation OID and tablespace reads

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
  - result: direct unsafe count is `1278`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `relation-oid-helper-scan.log`
  - command: `rg -n 'unsafe \{ crate::storage::relation::relation_oid|unsafe \{ crate::storage::relation::relation_tablespace|relation_oid_handle|relation_tablespace_handle' src/storage/relation.rs src/am/ec_ivf/insert.rs src/am/ec_spire/insert.rs src/am/ec_spire/storage/relation_plan.rs src/am/ec_spire/storage/relation_store.rs src/am/ec_spire/coordinator/snapshots.rs`
  - result: updated call sites use checked handle helpers; no targeted caller-side `unsafe { relation_oid(...) }` / `unsafe { relation_tablespace(...) }` wrappers remain in these files
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/353-relation-oid-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/353-relation-oid-nonnull-helpers src`
  - result: wrote `1278` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/353-relation-oid-nonnull-helpers/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1278 current unsafe rows`
