# Artifact Manifest

- Packet: `reviews/task-50/367-spire-delete-tuple-length-visitor`
- Code commit: `175b62e3cd5c7a635108264fff989a472219321e`
- Timestamp: `2026-05-22T01:32:11Z`
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
- Result: `1201`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches
- Note: `rg` returned exit code 1 because no rows matched.

### `spire-delete-helper-scan.log`

- Command: `rg -n 'object tuple delete|PageGetItemId\\(|lp_flags|lp_len|visit_tuple_bytes' src/am/ec_spire/page.rs src/storage/buffer_guard.rs`
- Result: SPIRE object delete tuple length now comes through
  `LockedBufferGuard::visit_tuple_bytes`; line-pointer reads are in the shared
  visitor.

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/367-spire-delete-tuple-length-visitor/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/367-spire-delete-tuple-length-visitor src`
- Result: `1201` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/367-spire-delete-tuple-length-visitor/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: passed
- Key line: `ledger covers 1201 current unsafe rows`
