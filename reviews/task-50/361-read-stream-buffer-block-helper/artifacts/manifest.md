# Artifact Manifest: Task 50 Packet 361

- Head SHA: `9c2407a69b67f421a06c1da265f39ef5c2da1a8f`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/361-read-stream-buffer-block-helper/`
- Timestamp: `2026-05-21T18:03:41-07:00`
- Lane: unsafe burndown, P9 read-stream contracts
- Fixture/storage/rerank mode: not applicable
- Surface isolation: source-only validation, no benchmark storage surfaces

## Artifacts

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Key lines: `Finished dev profile ... target(s) in 13.95s`
- Notes: reports the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `unsafe-count.log`

- Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
- Result: `1218`

### `raw-boundary-guard.log`

- Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Result: no matches

### `read-stream-buffer-block-scan.log`

- Command: `rg -n 'per_buffer_data\\.cast::<pg_sys::BlockNumber>|read_stream_per_buffer_block_number' src/am/common/stream.rs`
- Result: one helper-owned `per_buffer_data.cast::<pg_sys::BlockNumber>()` read and two helper call sites

### `unsafe-ledger-after.jsonl`

- Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/361-read-stream-buffer-block-helper/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/361-read-stream-buffer-block-helper src`
- Result: wrote `1218` unsafe ledger rows

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/361-read-stream-buffer-block-helper/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Result: `ledger covers 1218 current unsafe rows`
