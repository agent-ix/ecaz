# Artifact Manifest

Task bucket: `reviews/task-50/370-ivf-page-relation-nonnull-view`

Head SHA: `59fad9a35ada90ffb67ad9e804c09fef51a1ed14`

Timestamp: `2026-05-21T18:51:32-07:00`

Lane: Task 50 unsafe burndown, IVF page relation view / page tuple contract.

Fixture/storage/rerank mode: static compile and ledger validation only; no
runtime benchmark or corpus fixture was needed because this slice changes the
relation view type and private helper plumbing without changing tuple bytes,
candidate ordering, WAL order, scoring math, or allocation shape.

## Artifacts

### `code-diff.patch`

- Command: `git show --format= --patch HEAD -- src/am/ec_ivf/page.rs`
- Key result: code diff for commit
  `59fad9a35ada90ffb67ad9e804c09fef51a1ed14`.

### `unsafe-counts.log`

- Command:
  `git show HEAD^:src/am/ec_ivf/page.rs | rg -n 'unsafe\s*\{' | wc -l; rg -n 'unsafe\s*\{' src/am/ec_ivf/page.rs | wc -l; rg -n 'unsafe\s*\{' src | wc -l`
- Key result lines:
  - `29`
  - `18`
  - `1187`
- Interpretation: touched file direct unsafe count changed `29 -> 18`; current
  `src/` direct unsafe count is `1187`.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: `Finished dev profile`.
- Note: reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: command exit code `0`.

### `rustfmt-page-check.log`

- Command: `rustfmt --check src/am/ec_ivf/page.rs`
- Key result: command exit code `0`.
- Note: rustfmt reports existing stable-toolchain warnings for unstable
  rustfmt options.

### `raw-boundary-guard.log`

- Command:
  `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Key result: no matches; `rg` exit code `1` is expected for an empty result.

### `unsafe-ledger-after.jsonl`

- Command:
  `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/370-ivf-page-relation-nonnull-view/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/370-ivf-page-relation-nonnull-view src`
- Key result: `wrote 1187 unsafe ledger rows`.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Key result: `wrote 1187 unsafe ledger rows`.

### `unsafe-ledger-check.log`

- Command:
  `make UNSAFE_LEDGER=reviews/task-50/370-ivf-page-relation-nonnull-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1187 current unsafe rows`.

### `cargo-fmt-check.log`

- Command: `cargo fmt --all -- --check`
- Note: retained for transparency but not used as acceptance evidence. The
  workspace-wide command reports unrelated pre-existing formatting diffs outside
  this slice. The slice-specific `rustfmt-page-check.log` passed for the touched
  file.
