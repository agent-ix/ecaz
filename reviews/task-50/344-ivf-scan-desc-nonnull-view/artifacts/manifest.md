# Task 50 Packet 344 Artifact Manifest

- head SHA: `657787920342830011f325c96f361db23a70cd79`
- parent SHA: `0cad0e332db1965df8a304c4b5916eb07f2efadc`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/344-ivf-scan-desc-nonnull-view/`
- timestamp: `2026-05-21T23:42:49Z`
- lane: Task 50 unsafe burndown, IVF scan descriptor view cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF AM scan descriptor callback/debug helpers

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
  - result: direct unsafe count is `1308`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `ivf-scan-desc-helper-scan.log`
  - command: `rg -n 'IvfScanDescView::from_raw|unsafe \{ IvfScanDescView::from_raw' src/am/ec_ivf/scan.rs`
  - result: no hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/344-ivf-scan-desc-nonnull-view/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/344-ivf-scan-desc-nonnull-view src`
  - result: wrote `1308` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/344-ivf-scan-desc-nonnull-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1308 current unsafe rows`
