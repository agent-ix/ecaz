# Task 50 Packet 345 Artifact Manifest

- head SHA: `ba24c978ad50ffc17f595c385c367d075be18483`
- parent SHA: `97100fc1133f7bb82578578f6bca8f6e5d0a2fa2`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/345-ivf-index-heap-oid-nonnull-relation/`
- timestamp: `2026-05-21T23:46:18Z`
- lane: Task 50 unsafe burndown, IVF relation metadata handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF scan descriptor heap OID resolution and debug relation lookup

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
  - result: direct unsafe count is `1306`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `ivf-index-heap-oid-scan.log`
  - command: `rg -n 'unsafe \{ ivf_index_heap_oid|ivf_index_heap_oid\(index_relation,' src/am/ec_ivf/scan.rs`
  - result: no hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/345-ivf-index-heap-oid-nonnull-relation/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/345-ivf-index-heap-oid-nonnull-relation src`
  - result: wrote `1306` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/345-ivf-index-heap-oid-nonnull-relation/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1306 current unsafe rows`
