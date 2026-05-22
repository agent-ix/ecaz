# Artifact Manifest: Task 50 Packet 357

- Head SHA: `f4556dee11bbaa95fa7db8faec7134495181efff`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/357-relation-raw-wrapper-closeout/`
- Timestamp: `2026-05-21T17:45:49-07:00`
- Lane: unsafe burndown, P2 PostgreSQL relation handle views
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench-clean.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 13.41s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check-clean.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count-clean.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1228`

### `raw-boundary-guard-clean.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `relation-raw-wrapper-call-scan-clean.log`

- Command: `rg -n 'crate::storage::relation::(main_fork_block_count|relation_oid|index_heap_relation_oid)\\(' src/am src/lib.rs src/tests src/storage`
- Result: no matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/357-relation-raw-wrapper-closeout/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/357-relation-raw-wrapper-closeout src`
- Result: wrote `1228` unsafe ledger rows

### `unsafe-ledger-check-clean.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/357-relation-raw-wrapper-closeout/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1228 current unsafe rows`

## Trace Logs

Earlier logs are retained for reviewer traceability:

- `cargo-check-pg18-bench.log`: initial compile failed because SPIRE snapshot storage relation used a local live relation view, not `RelationGuard`.
- `cargo-check-pg18-bench-final.log`: compile passed but reported new dead warnings after the raw wrappers became unused.
- `git-diff-check.log`, `git-diff-check-final.log`, `unsafe-count.log`, `unsafe-count-final.log`, `raw-boundary-guard.log`, `raw-boundary-guard-final.log`, `relation-raw-wrapper-call-scan.log`, `relation-raw-wrapper-call-scan-final.log`, `unsafe-ledger-check.log`, and `unsafe-ledger-check-final.log`: intermediate validations before final dead-wrapper cleanup.
