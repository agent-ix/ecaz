# Task 50 Packet 377 Artifact Manifest

- head SHA: `36770634cb733b8a9f10ad1cf42c0ab51395d7ec`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/377-spire-relation-plan-boundaries`
- timestamp: `2026-05-22T02:20:31Z`
- lane: Task 50 unsafe burndown, SPIRE relation-plan boundary consolidation
- fixture/storage/rerank: SPIRE auxiliary local-store relation planning; no runtime fixture
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - key lines: `Finished dev profile`; known pre-existing warning remains in `src/am/mod.rs` for unused SPIRE DML re-exports.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed with no output.
- `rustfmt-relation-plan-check.log`
  - command: `rustfmt --check src/am/ec_spire/storage/relation_plan.rs`
  - result: passed; stable rustfmt emitted the known warnings about ignored nightly-only import grouping options.
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no matches.
- `relation-plan-unsafe-counts.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/storage/relation_plan.rs`
  - result: 5 remaining direct unsafe blocks in `src/am/ec_spire/storage/relation_plan.rs`.
- `src-unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: `1169`
- `unsafe-ledger-generate.log`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/377-spire-relation-plan-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/377-spire-relation-plan-boundaries src`
  - result: wrote `1169` ledger rows.
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/377-spire-relation-plan-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: `ledger covers 1169 current unsafe rows`
- `unsafe-ledger-after.jsonl`
  - generated unsafe ledger for the post-slice tree.
- `code-diff.patch`
  - command: `git show --format= --no-color HEAD -- src/am/ec_spire/storage/relation_plan.rs`
  - result: code diff for commit `36770634cb733b8a9f10ad1cf42c0ab51395d7ec`.
