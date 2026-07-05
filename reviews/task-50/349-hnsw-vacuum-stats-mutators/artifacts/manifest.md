# Task 50 Packet 349 Artifact Manifest

- head SHA: `1fb2c405e8bf6b570de46d57353bd481955d293e`
- parent SHA: `ef3849b789a1387f7923169a299d68a05f4208c9`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/349-hnsw-vacuum-stats-mutators/`
- timestamp: `2026-05-21T23:58:56Z`
- lane: Task 50 unsafe burndown, HNSW vacuum stats mutation cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: HNSW noop vacuum stats and bulkdelete stats result mutation

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
  - result: direct unsafe count is `1301`
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no hits
- `hnsw-vacuum-stats-scan.log`
  - command: `rg -n '\(\*stats\)\.|unsafe \{\s*\(\*stats\)' src/am/ec_hnsw/shared.rs src/am/ec_hnsw/vacuum.rs src/am/common/vacuum.rs`
  - result: no direct HNSW/common `(*stats)` field-write hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/349-hnsw-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/349-hnsw-vacuum-stats-mutators src`
  - result: wrote `1301` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/349-hnsw-vacuum-stats-mutators/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1301 current unsafe rows`
