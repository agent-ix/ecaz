# Task 50 Packet 384 Artifact Manifest

- head SHA: `06b8c36c3f1aad5faf385a302f59005102c50af0`
- task bucket: `reviews/task-50`
- packet: `reviews/task-50/384-spire-registered-page-operations`
- lane: SPIRE unsafe burndown
- fixture/storage/rerank: not applicable; code-level page helper consolidation
- timestamp: `2026-05-21T19:46:57-07:00`
- table surface: not applicable; no benchmark or SQL fixture

## Artifacts

### `cargo-check-pg18-bench.log`

- command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/384-spire-registered-page-operations/artifacts/cargo-check-pg18-bench.log`
- result: passed
- key lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 28.25s`
- note: warning is the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `rustfmt-page.log`

- command: `script -q -c "rustfmt --check src/am/ec_spire/page.rs" reviews/task-50/384-spire-registered-page-operations/artifacts/rustfmt-page.log`
- result: passed
- key lines:
  - rustfmt emitted the existing stable-toolchain warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- command: `script -q -c "git diff --check" reviews/task-50/384-spire-registered-page-operations/artifacts/git-diff-check.log`
- result: passed

### `raw-boundary-guard.log`

- command: `script -q -c "rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src" reviews/task-50/384-spire-registered-page-operations/artifacts/raw-boundary-guard.log`
- result: passed; no matches

### `src-unsafe-count.log`

- command: `script -q -c "rg -n 'unsafe\\s*\\{' src | wc -l" reviews/task-50/384-spire-registered-page-operations/artifacts/src-unsafe-count.log`
- result: `1154`

### `unsafe-ledger-generate.log`

- command: `script -q -c "python3 scripts/unsafe_ledger.py generate --output reviews/task-50/384-spire-registered-page-operations/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/384-spire-registered-page-operations src" reviews/task-50/384-spire-registered-page-operations/artifacts/unsafe-ledger-generate.log`
- result: passed
- key line: `wrote 1154 unsafe ledger rows to reviews/task-50/384-spire-registered-page-operations/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/384-spire-registered-page-operations/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/384-spire-registered-page-operations/artifacts/unsafe-ledger-check.log`
- result: passed
- key line: `ledger covers 1154 current unsafe rows`

### `unsafe-ledger-after.jsonl`

- command output from unsafe ledger generation
- result: generated ledger for the post-change source tree
