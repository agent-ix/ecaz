# Task 50 Packet 382 Artifact Manifest

- head SHA: `fcd544d8e`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/382-spire-diagnostics-manifest-boundary`
- timestamp: `2026-05-22T02:37:37Z`
- lane: Task 50 unsafe burndown, SPIRE diagnostics manifest read boundary
- fixture/storage/rerank: SPIRE boundary-replica diagnostics manifest read path; no runtime fixture
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - key lines: `Finished dev profile`; known pre-existing warning remains in `src/am/mod.rs` for unused SPIRE DML re-exports.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passed with no output.
- `rustfmt-diagnostics-check.log`
  - command: `rustfmt --check src/am/ec_spire/coordinator/diagnostics.rs`
  - result: passed; stable rustfmt emitted the known warnings about ignored nightly-only import grouping options.
- `raw-boundary-guard.log`
  - command: `rg -n '^pub(\(crate\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - result: no matches.
- `diagnostics-unsafe-counts.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/coordinator/diagnostics.rs`
  - result: 1 remaining direct unsafe block in `src/am/ec_spire/coordinator/diagnostics.rs`.
- `src-unsafe-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - result: `1156`
- `unsafe-ledger-generate.log`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/382-spire-diagnostics-manifest-boundary/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/382-spire-diagnostics-manifest-boundary src`
  - result: wrote `1156` ledger rows.
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/382-spire-diagnostics-manifest-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: `ledger covers 1156 current unsafe rows`
- `unsafe-ledger-after.jsonl`
  - generated unsafe ledger for the post-slice tree.
- `code-diff.patch`
  - command: `git diff -- src/am/ec_spire/coordinator/diagnostics.rs` before the branch advanced to `fcd544d8e`
  - result: code diff for the diagnostics consolidation now present in `fcd544d8e`.
