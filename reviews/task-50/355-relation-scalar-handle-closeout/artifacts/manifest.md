# Artifact Manifest: Task 50 Packet 355

- Head SHA: `6c13902b12784d9d370cd15e7be03ec3f51ed3c6`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/355-relation-scalar-handle-closeout/`
- Timestamp: `2026-05-21T17:30:19-07:00`
- Lane: unsafe burndown, relation scalar handle closeout
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench-final.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 16.09s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check-final.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count-final.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1246`

### `raw-boundary-guard-final.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `relation-scalar-targeted-scan-final.log`

- Command: `rg -n 'unsafe \\{ crate::storage::relation::(main_fork_block_count|relation_reltuples|relation_am_oid)' src/am/common src/am/ec_diskann src/am/ec_hnsw src/am/ec_spire`
- Result: no matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/355-relation-scalar-handle-closeout/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/355-relation-scalar-handle-closeout src`
- Result: wrote `1246` unsafe ledger rows

### `unsafe-ledger-check-final.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/355-relation-scalar-handle-closeout/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1246 current unsafe rows`

## Trace Logs

Earlier run logs are retained in this packet for reviewer traceability:

- `cargo-check-pg18-bench.log`: initial compile failed on a DiskANN raw/handle shadowing error introduced in this slice.
- `cargo-check-pg18-bench-rerun.log`: compile passed after the shadowing fix but exposed a new unused raw `relation_reltuples` wrapper warning.
- `git-diff-check.log`, `git-diff-check-rerun.log`, `unsafe-count.log`, `unsafe-count-rerun.log`, `raw-boundary-guard.log`, `raw-boundary-guard-rerun.log`, `relation-scalar-targeted-scan.log`, `relation-scalar-targeted-scan-rerun.log`, and `unsafe-ledger-check.log`: intermediate successful validations before final wrapper cleanup.
