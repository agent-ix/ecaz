# Task 50 Packet 346 Artifact Manifest

- head SHA: `44eabec15aebb93c6f0fcde141ade80b4203f83b`
- parent SHA: `8f18b15a3821a498dc548eb04067f8a66e8ca2e7`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/346-ivf-reloptions-nonnull-relation/`
- timestamp: `2026-05-21T23:49:42Z`
- lane: Task 50 unsafe burndown, IVF reloptions relation-handle cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: IVF relation options for build, build-empty, admin snapshot, and scan rescan

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
  - result: direct unsafe count is `1305`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `ivf-reloptions-scan.log`
  - command: `rg -n 'relation_options\(([^N]|$)|EcIvfReloptionsView::from_relation|unsafe \{ EcIvfReloptionsView::from_relation' src/am/ec_ivf`
  - result: all IVF `options::relation_options` callers pass checked handles; `EcIvfReloptionsView::from_relation` has no caller-side unsafe wrapper
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/346-ivf-reloptions-nonnull-relation/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/346-ivf-reloptions-nonnull-relation src`
  - result: wrote `1305` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/346-ivf-reloptions-nonnull-relation/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1305 current unsafe rows`
