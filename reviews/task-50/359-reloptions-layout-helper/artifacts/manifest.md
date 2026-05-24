# Artifact Manifest: Task 50 Packet 359

- Head SHA: `1695e4dafc5f7becf701ee76d7cc1ca5bf752a70`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/359-reloptions-layout-helper/`
- Timestamp: `2026-05-21T17:55:58-07:00`
- Lane: unsafe burndown, P7 reloptions contracts
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 15.96s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1223`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `reloptions-cast-scan.log`

- Command: `rg -n 'let reloptions = unsafe \\{ &\\*rd_options\\.cast::<|PhantomData|reloptions: &'\'' ' src/am/ec_diskann/options.rs src/am/ec_hnsw/options.rs src/am/ec_ivf/options.rs src/am/ec_spire/options/mod.rs`
- Result: no matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/359-reloptions-layout-helper/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/359-reloptions-layout-helper src`
- Result: wrote `1223` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/359-reloptions-layout-helper/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1223 current unsafe rows`
