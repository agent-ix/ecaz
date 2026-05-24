# Task 50 Packet 347 Artifact Manifest

- head SHA: `68719357478897e7fb7abf95e5dd0ac4b7cfea0b`
- parent SHA: `26aa7dd76d083296d22dac2331f279a13da06584`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/347-ivf-vacuum-stats-mutators/`
- timestamp: `2026-05-21T23:52:30Z`
- lane: Task 50 unsafe burndown, IVF vacuum stats mutation cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF vacuum bulkdelete and vacuumcleanup stats result mutation

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
  - result: direct unsafe count is `1304`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `ivf-vacuum-stats-scan.log`
  - command: `rg -n '\(\*stats\)\.|unsafe \{\s*\(\*stats\)' src/am/ec_ivf/vacuum.rs src/am/common/vacuum.rs`
  - result: no direct IVF/common `(*stats)` field-write hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/347-ivf-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/347-ivf-vacuum-stats-mutators src`
  - result: wrote `1304` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/347-ivf-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1304 current unsafe rows`
