# Artifact Manifest

Task bucket: `reviews/task-50/372-explain-option-state-read`

Head SHA: `58dbb7e340e8fe912594988cc24844e1b4291847`

Timestamp: `2026-05-21T19:00:39-07:00`

Lane: Task 50 unsafe burndown, AM common EXPLAIN option state cleanup.

Fixture/storage/rerank mode: static compile and ledger validation only; no
runtime benchmark or corpus fixture was needed because this slice only
coalesces the EXPLAIN extension-state raw pointer read and does not change
planner output semantics.

## Artifacts

### `code-diff.patch`

- Command: `git show --format= --patch HEAD -- src/am/common/explain.rs`
- Key result: code diff for commit
  `58dbb7e340e8fe912594988cc24844e1b4291847`.

### `unsafe-counts.log`

- Command:
  `git show HEAD^:src/am/common/explain.rs | rg -n 'unsafe\s*\{' | wc -l; rg -n 'unsafe\s*\{' src/am/common/explain.rs | wc -l; rg -n 'unsafe\s*\{' src | wc -l`
- Key result lines:
  - `12`
  - `11`
  - `1183`
- Interpretation: touched file count changed `12 -> 11`; current `src/`
  direct unsafe count is `1183`.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: `Finished dev profile`.
- Note: reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: command exit code `0`.

### `rustfmt-explain-check.log`

- Command: `rustfmt --check src/am/common/explain.rs`
- Key result: command exit code `0`.
- Note: rustfmt reports existing stable-toolchain warnings for unstable
  rustfmt options.

### `raw-boundary-guard.log`

- Command:
  `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Key result: no matches; `rg` exit code `1` is expected for an empty result.

### `unsafe-ledger-after.jsonl`

- Command:
  `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/372-explain-option-state-read/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/372-explain-option-state-read src`
- Key result: `wrote 1183 unsafe ledger rows`.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Key result: `wrote 1183 unsafe ledger rows`.

### `unsafe-ledger-check.log`

- Command:
  `make UNSAFE_LEDGER=reviews/task-50/372-explain-option-state-read/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1183 current unsafe rows`.
