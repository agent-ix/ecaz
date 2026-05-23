# Task 50 Packet 348 Artifact Manifest

- head SHA: `787e4860f35e213d4bb18172068a4d7ab9cb3278`
- parent SHA: `a4b1d13e5d528f3d7c4256a77d122c4c7a30f547`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/348-spire-vacuum-stats-mutators/`
- timestamp: `2026-05-21T23:56:06Z`
- lane: Task 50 unsafe burndown, SPIRE vacuum stats mutation cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: SPIRE vacuum bulkdelete and vacuumcleanup stats result mutation

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
  - result: direct unsafe count is `1303`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `spire-vacuum-stats-scan.log`
  - command: `rg -n '\(\*stats\)\.|unsafe \{\s*\(\*stats\)' src/am/ec_spire/vacuum/mod.rs src/am/common/vacuum.rs`
  - result: no direct SPIRE/common `(*stats)` field-write hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/348-spire-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/348-spire-vacuum-stats-mutators src`
  - result: wrote `1303` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/348-spire-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1303 current unsafe rows`
