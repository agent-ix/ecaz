# Artifact Manifest

Task bucket: `reviews/task-50/373-stringinfo-reader-boundaries`

Head SHA: `5660ef507439d91d86f7ffd5fd0cd0fd0cfd96f6`

Timestamp: `2026-05-21T19:03:34-07:00`

Lane: Task 50 unsafe burndown, `StringInfoReader` receive-buffer boundary
cleanup.

Fixture/storage/rerank mode: static compile and ledger validation only; no
runtime benchmark or corpus fixture was needed because this slice only
coalesces receive-buffer boundary operations and preserves the reader API.

## Artifacts

### `code-diff.patch`

- Command: `git show --format= --patch HEAD -- src/storage/string_info.rs`
- Key result: code diff for commit
  `5660ef507439d91d86f7ffd5fd0cd0fd0cfd96f6`.

### `unsafe-counts.log`

- Command:
  `git show HEAD^:src/storage/string_info.rs | rg -n 'unsafe\s*\{' | wc -l; rg -n 'unsafe\s*\{' src/storage/string_info.rs | wc -l; rg -n 'unsafe\s*\{' src | wc -l`
- Key result lines:
  - `7`
  - `4`
  - `1180`
- Interpretation: touched file count changed `7 -> 4`; current `src/` direct
  unsafe count is `1180`.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: `Finished dev profile`.
- Note: reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: command exit code `0`.

### `rustfmt-string-info-check.log`

- Command: `rustfmt --check src/storage/string_info.rs`
- Key result: command exit code `0`.
- Note: rustfmt reports existing stable-toolchain warnings for unstable
  rustfmt options.

### `raw-boundary-guard.log`

- Command:
  `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Key result: no matches; `rg` exit code `1` is expected for an empty result.

### `unsafe-ledger-after.jsonl`

- Command:
  `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/373-stringinfo-reader-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/373-stringinfo-reader-boundaries src`
- Key result: `wrote 1180 unsafe ledger rows`.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Key result: `wrote 1180 unsafe ledger rows`.

### `unsafe-ledger-check.log`

- Command:
  `make UNSAFE_LEDGER=reviews/task-50/373-stringinfo-reader-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1180 current unsafe rows`.
