# Artifact Manifest

- Packet: `reviews/task-50/363-spire-object-tuple-visitor`
- Code commit: `e1bf25f1c0311c447d92fdb44535a96c561a4d66`
- Timestamp: `2026-05-22T01:14:43Z`
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
- Result: `1210`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `spire-object-tuple-helper-scan.log`

- Command: `rg -n 'with_object_tuple_from_locked_page|visit_object_tuple_from_locked_page|SpireObjectTupleVisit|with_wal_object_tuple_mut|PageGetItemId\\(page, tid\\.offset_number\\)|PageGetItem\\(page, item_id\\)' src/am/ec_spire/page.rs src/storage/buffer_guard.rs`
- Result: deleted the immutable SPIRE visitor; remaining matches are the shared
  locked-buffer tuple boundary and the WAL/exclusive mutable rewrite helper.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/363-spire-object-tuple-visitor/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/363-spire-object-tuple-visitor src`
- Result: `1210` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/363-spire-object-tuple-visitor/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1210 current unsafe rows`
