# Artifact Manifest

- Packet: `reviews/task-50/366-spire-wal-tuple-visitor`
- Code commit: `0c4e9f528f98066400e0f04a04d603cb16372a01`
- Timestamp: `2026-05-22T01:28:59Z`
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
- Result: `1202`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `spire-wal-helper-scan.log`

- Command: `rg -n 'with_wal_object_tuple_mut|register_locked_buffer_full_image_page|from_raw_parts_mut\\(|PageGetItemId\\(|PageGetItem\\(' src/am/ec_spire/page.rs src/storage/wal.rs`
- Result: SPIRE-local mutable object tuple helper removed; remaining mutable
  tuple pointer operations are centralized in the WAL registered-page token.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/366-spire-wal-tuple-visitor/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/366-spire-wal-tuple-visitor src`
- Result: `1202` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/366-spire-wal-tuple-visitor/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1202 current unsafe rows`
