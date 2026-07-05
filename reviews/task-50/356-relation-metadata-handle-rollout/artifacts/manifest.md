# Artifact Manifest: Task 50 Packet 356

- Head SHA: `39e8de240b32d19ef54309c3c96fb483a62497ad`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/356-relation-metadata-handle-rollout/`
- Timestamp: `2026-05-21T17:38:38-07:00`
- Lane: unsafe burndown, P2 PostgreSQL handle views
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench-final.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 14.14s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check-final.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count-final.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1230`

### `raw-boundary-guard-final.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `relation-metadata-raw-call-scan-final.log`

- Command: `rg -n 'crate::storage::relation::relation_(name|kind|am_oid|namespace_owner_persistence|tuple_desc_copy|raw_tuple_desc_copy|options|tablespace)\\(' src`
- Result: no matches

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/356-relation-metadata-handle-rollout/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/356-relation-metadata-handle-rollout src`
- Result: wrote `1230` unsafe ledger rows

### `unsafe-ledger-check-final.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/356-relation-metadata-handle-rollout/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1230 current unsafe rows`

## Trace Logs

Earlier logs are retained for reviewer traceability:

- `cargo-check-pg18-bench.log`: initial compile passed but included a dead-code warning for `IndexRelationGuard::handle()` in normal builds.
- `git-diff-check.log`, `unsafe-count.log`, `raw-boundary-guard.log`, `relation-metadata-raw-call-scan.log`, and `unsafe-ledger-check.log`: intermediate successful validations before the final test/pg_test gating adjustment.
