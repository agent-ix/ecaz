# Artifact Manifest

- Packet: `reviews/task-50/368-stringinfo-receive-reader`
- Code commit: `9165f2e4d12acab0e4736364dcf944670fd06bd2`
- Timestamp: `2026-05-22T01:37:17Z`
- Lane: Task 50 unsafe burndown, P1 receive-buffer boundary contracts
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
- Result: `1199`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `stringinfo-receive-scan.log`

- Command: `rg -n 'recv_tqvector_message|recv_raw_f32_message|StringInfoReader::from_raw|from_internal|StringInfoData|pg_sys::StringInfo' src/lib.rs src/storage/string_info.rs`
- Result: root receive functions use `StringInfoReader::from_internal`; raw
  `pg_sys::StringInfo` handling is centralized in `src/storage/string_info.rs`.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/368-stringinfo-receive-reader/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/368-stringinfo-receive-reader src`
- Result: `1199` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/368-stringinfo-receive-reader/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1199 current unsafe rows`
