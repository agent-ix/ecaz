# Task 50 Packet 379 Artifact Manifest

- head SHA: `1548a3e7ba30d7b71059d3225495a545a816785d`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/379-spire-hierarchy-boundaries`
- timestamp: `2026-05-22T02:27:24Z`
- lane: Task 50 unsafe burndown, SPIRE hierarchy snapshot boundaries
- fixture/storage/rerank: SPIRE coordinator remote/local heap candidate and manifest read paths; no runtime fixture
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - key lines: `Finished dev profile`; known pre-existing warning remains in `src/am/mod.rs` for unused SPIRE DML re-exports.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed with no output.
- `rustfmt-hierarchy-snapshots-check.log`
  - command: `rustfmt --check src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
  - result: passed; stable rustfmt emitted the known warnings about ignored nightly-only import grouping options.
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no matches.
- `hierarchy-snapshots-unsafe-counts.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
  - result: 2 remaining direct unsafe blocks in `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`.
- `src-unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: `1164`
- `unsafe-ledger-generate.log`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/379-spire-hierarchy-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/379-spire-hierarchy-boundaries src`
  - result: wrote `1164` ledger rows.
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/379-spire-hierarchy-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: `ledger covers 1164 current unsafe rows`
- `unsafe-ledger-after.jsonl`
  - generated unsafe ledger for the post-slice tree.
- `code-diff.patch`
  - command: `git show --format= --no-color HEAD -- src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
  - result: code diff for commit `1548a3e7ba30d7b71059d3225495a545a816785d`.
