# Task 50 Packet 380 Artifact Manifest

- head SHA: `b2a070479e8f1281c10a0cc74eb2a79cc2a5bd3c`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/380-spire-maintenance-publish-boundaries`
- timestamp: `2026-05-22T02:31:14Z`
- lane: Task 50 unsafe burndown, SPIRE maintenance publish boundaries
- fixture/storage/rerank: SPIRE maintenance and epoch cleanup publish-lock paths; no runtime fixture
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - key lines: `Finished dev profile`; known pre-existing warning remains in `src/am/mod.rs` for unused SPIRE DML re-exports.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed with no output.
- `rustfmt-spire-maintenance-snapshots-check.log`
  - command: `rustfmt --check src/am/ec_spire/coordinator/maintenance.rs src/am/ec_spire/coordinator/snapshots.rs`
  - result: passed; stable rustfmt emitted the known warnings about ignored nightly-only import grouping options.
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no matches.
- `spire-maintenance-snapshots-unsafe-counts.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/coordinator/maintenance.rs src/am/ec_spire/coordinator/snapshots.rs`
  - result: remaining direct unsafe blocks in the two touched files are listed; no raw publish-lock callsites remain outside `SpireLiveIndexRelation::publish_lock()` in these files.
- `src-unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: `1159`
- `unsafe-ledger-generate.log`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/380-spire-maintenance-publish-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/380-spire-maintenance-publish-boundaries src`
  - result: wrote `1159` ledger rows.
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/380-spire-maintenance-publish-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: `ledger covers 1159 current unsafe rows`
- `unsafe-ledger-after.jsonl`
  - generated unsafe ledger for the post-slice tree.
- `code-diff.patch`
  - command: `git show --format= --no-color HEAD -- src/am/ec_spire/coordinator/maintenance.rs src/am/ec_spire/coordinator/snapshots.rs`
  - result: code diff for commit `b2a070479e8f1281c10a0cc74eb2a79cc2a5bd3c`.
