# Artifact Manifest

- Packet: `reviews/task-50/362-locked-buffer-tuple-visitor`
- Code commit: `b534684272310df5dffe06cd5f54deb4d0835ce3`
- Timestamp: `2026-05-22T01:11:05Z`
- Lane: Task 50 unsafe burndown, P4 page tuple / line-pointer contracts
- Fixture/storage/rerank: not applicable; source-level structural cleanup
- Isolation surface: not applicable; no benchmark run

## Artifacts

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key line: `Finished dev profile`
- Note: reports the known pre-existing unused SPIRE DML re-export warning in
  `src/am/mod.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `src-unsafe-count-after.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1213`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `diskann-tuple-helper-scan.log`

- Command: `rg -n 'copy_data_page_tuple_bytes|with_vacuum_page_tuple_bytes\\(|PageGetItemId\\(page, offset\\)|PageGetItem\\(page, item_id\\)' src/am/ec_diskann/scan_state.rs src/am/ec_diskann/routine.rs src/storage/buffer_guard.rs`
- Result: only the centralized `PageGetItem` boundary in
  `src/storage/buffer_guard.rs` remains.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/362-locked-buffer-tuple-visitor/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/362-locked-buffer-tuple-visitor src`
- Result: `1213` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/362-locked-buffer-tuple-visitor/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1213 current unsafe rows`
