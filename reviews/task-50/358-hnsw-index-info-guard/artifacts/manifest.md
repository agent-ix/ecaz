# Artifact Manifest: Task 50 Packet 358

- Head SHA: `c5469136ddff78cf7c5d46c9b73ebce2e3674388`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/358-hnsw-index-info-guard/`
- Timestamp: `2026-05-21T17:49:59-07:00`
- Lane: unsafe burndown, P5 heap/source metadata
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 25.71s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1226`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `hnsw-index-info-scan.log`

- Command: `rg -n 'pfree\\(index_info|BuildIndexInfo\\(' src/am/ec_hnsw/source.rs`
- Result: one `BuildIndexInfo` boundary inside `IndexInfoGuard::build`; no manual `pfree(index_info...)` matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/358-hnsw-index-info-guard/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/358-hnsw-index-info-guard src`
- Result: wrote `1226` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/358-hnsw-index-info-guard/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1226 current unsafe rows`
