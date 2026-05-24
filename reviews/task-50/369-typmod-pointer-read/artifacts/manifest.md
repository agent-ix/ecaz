# Artifact Manifest

- Packet: `reviews/task-50/369-typmod-pointer-read`
- Code commit: `e2279726d5cd24b5132a4ecabee2e57e26cafd27`
- Timestamp: `2026-05-22T01:40:31Z`
- Lane: Task 50 unsafe burndown, P1 root receive / typmod boundary contracts
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
- Result: `1198`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `typmod-pointer-scan.log`

- Command: `rg -n 'ArrayGetIntegerTypmods|raw_typmods|single_typmod|invalid type modifier' src/lib.rs`
- Result: the typmod pointer read is guarded by a count check and null check
  inside `DetoastedTypmodArray::single_typmod`.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/369-typmod-pointer-read/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/369-typmod-pointer-read src`
- Result: `1198` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/369-typmod-pointer-read/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1198 current unsafe rows`
