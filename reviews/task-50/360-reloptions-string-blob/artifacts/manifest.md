# Artifact Manifest: Task 50 Packet 360

- Head SHA: `aa4bacec4d93c37264b3d1699ed102e163c02e66`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/360-reloptions-string-blob/`
- Timestamp: `2026-05-21T18:00:37-07:00`
- Lane: unsafe burndown, P7 reloptions contracts
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 23.23s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1219`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `reloptions-string-scan.log`

- Command: `rg -n 'unsafe \\{[^\n]*read_string_reloption|read_string_reloption\\(.*rd_options' src/am/ec_diskann/options.rs src/am/ec_hnsw/options.rs src/am/ec_ivf/options.rs src/am/ec_spire/options/mod.rs src/am/common/reloptions.rs`
- Result: no matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/360-reloptions-string-blob/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/360-reloptions-string-blob src`
- Result: wrote `1219` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/360-reloptions-string-blob/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1219 current unsafe rows`
