# Task 50 Packet 381 Artifact Manifest

- head SHA: `c51bf73758157f45690d6731973e4090fe21caf4`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/381-spire-maintenance-execution-boundary`
- timestamp: `2026-05-22T02:33:53Z`
- lane: Task 50 unsafe burndown, SPIRE maintenance publish execution
- fixture/storage/rerank: SPIRE scheduled replacement maintenance publish path; no runtime fixture
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - key lines: `Finished dev profile`; known pre-existing warning remains in `src/am/mod.rs` for unused SPIRE DML re-exports.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed with no output.
- `rustfmt-maintenance-check.log`
  - command: `rustfmt --check src/am/ec_spire/coordinator/maintenance.rs`
  - result: passed; stable rustfmt emitted the known warnings about ignored nightly-only import grouping options.
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no matches.
- `maintenance-unsafe-counts.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/coordinator/maintenance.rs`
  - result: 3 remaining direct unsafe blocks in `src/am/ec_spire/coordinator/maintenance.rs`.
- `src-unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: `1158`
- `unsafe-ledger-generate.log`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/381-spire-maintenance-execution-boundary/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/381-spire-maintenance-execution-boundary src`
  - result: wrote `1158` ledger rows.
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/381-spire-maintenance-execution-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: `ledger covers 1158 current unsafe rows`
- `unsafe-ledger-after.jsonl`
  - generated unsafe ledger for the post-slice tree.
- `code-diff.patch`
  - command: `git show --format= --no-color HEAD -- src/am/ec_spire/coordinator/maintenance.rs`
  - result: code diff for commit `c51bf73758157f45690d6731973e4090fe21caf4`.
