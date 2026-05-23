# Task 50 Packet 350 Artifact Manifest

- head SHA: `9157526169f672ff67324d48a07095b16a9fe9ed`
- parent SHA: `5f4d0571167beeae7e24738f035169c7fe7e0636`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/350-diskann-vacuum-stats-mutators/`
- timestamp: `2026-05-22T00:01:34Z`
- lane: Task 50 unsafe burndown, DiskANN vacuum stats mutation cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: DiskANN noop vacuum stats and bulkdelete stats result mutation

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
  - result: direct unsafe count is `1299`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `diskann-vacuum-stats-scan.log`
  - command: `rg -n '\(\*stats\)\.|unsafe \{\s*\(\*stats\)' src/am/ec_diskann/routine.rs src/am/common/vacuum.rs`
  - result: no direct DiskANN/common `(*stats)` field-write hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/350-diskann-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/350-diskann-vacuum-stats-mutators src`
  - result: wrote `1299` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/350-diskann-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1299 current unsafe rows`
