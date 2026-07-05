# Artifact Manifest

Task bucket: `reviews/task-50/371-ivf-relation-scalar-handles`

Head SHA: `2e4bc4b1db5e5f59002beb5c4b7d416edd1ccca8`

Timestamp: `2026-05-21T18:57:26-07:00`

Lane: Task 50 unsafe burndown, IVF relation scalar handle cleanup.

Fixture/storage/rerank mode: static compile and ledger validation only; no
runtime benchmark or corpus fixture was needed because this slice only reroutes
relation scalar reads to existing shared helpers.

## Artifacts

### `code-diff.patch`

- Command:
  `git show --format= --patch HEAD -- src/am/ec_ivf/page.rs src/am/ec_ivf/scan.rs`
- Key result: code diff for commit
  `2e4bc4b1db5e5f59002beb5c4b7d416edd1ccca8`.

### `unsafe-counts.log`

- Command:
  `git show HEAD^:src/am/ec_ivf/page.rs | rg -n 'unsafe\s*\{' | wc -l; rg -n 'unsafe\s*\{' src/am/ec_ivf/page.rs | wc -l; git show HEAD^:src/am/ec_ivf/scan.rs | rg -n 'unsafe\s*\{' | wc -l; rg -n 'unsafe\s*\{' src/am/ec_ivf/scan.rs | wc -l; rg -n 'unsafe\s*\{' src | wc -l`
- Key result lines:
  - `18`
  - `16`
  - `24`
  - `23`
  - `1184`
- Interpretation: touched file counts changed `page.rs 18 -> 16` and
  `scan.rs 24 -> 23`; current `src/` direct unsafe count is `1184`.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: `Finished dev profile`.
- Note: reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: command exit code `0`.

### `rustfmt-ivf-check.log`

- Command: `rustfmt --check src/am/ec_ivf/page.rs src/am/ec_ivf/scan.rs`
- Key result: command exit code `0`.
- Note: rustfmt reports existing stable-toolchain warnings for unstable
  rustfmt options.

### `raw-boundary-guard.log`

- Command:
  `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
- Key result: no matches; `rg` exit code `1` is expected for an empty result.

### `unsafe-ledger-after.jsonl`

- Command:
  `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/371-ivf-relation-scalar-handles/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/371-ivf-relation-scalar-handles src`
- Key result: `wrote 1184 unsafe ledger rows`.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Key result: `wrote 1184 unsafe ledger rows`.

### `unsafe-ledger-check.log`

- Command:
  `make UNSAFE_LEDGER=reviews/task-50/371-ivf-relation-scalar-handles/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1184 current unsafe rows`.
