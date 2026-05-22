# Task 50 Packet 390 Artifact Manifest

- head SHA: `efc3da03a83cfdb9c18f86e7770a1102ce5c94d0`
- task bucket: `reviews/task-50`
- packet: `reviews/task-50/390-spire-vacuum-live-count-boundary`
- lane: SPIRE unsafe burndown
- fixture/storage/rerank: not applicable; code-level vacuum live-count boundary consolidation
- timestamp: `2026-05-21T20:12:59-07:00`
- table surface: not applicable; no benchmark or SQL fixture

## Artifacts

### `cargo-check-pg18-bench.log`

- command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/cargo-check-pg18-bench.log`
- result: passed
- key lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 14.36s`
- note: warning is the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `rustfmt-vacuum.log`

- command: `script -q -c "rustfmt --check src/am/ec_spire/vacuum/mod.rs" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/rustfmt-vacuum.log`
- result: passed
- key lines:
  - rustfmt emitted the existing stable-toolchain warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- command: `script -q -c "git diff --check" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/git-diff-check.log`
- result: passed

### `raw-boundary-guard.log`

- command: `script -q -c "rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/raw-boundary-guard.log`
- result: passed; no matches

### `src-unsafe-count.log`

- command: `script -q -c "rg -n 'unsafe\\s*\\{' src | wc -l" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/src-unsafe-count.log`
- result: `1134`

### `unsafe-ledger-generate.log`

- command: `script -q -c "python3 scripts/unsafe_ledger.py generate --output reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/390-spire-vacuum-live-count-boundary src" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/unsafe-ledger-generate.log`
- result: passed
- key line: `wrote 1134 unsafe ledger rows to reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/unsafe-ledger-check.log`
- result: passed
- key line: `ledger covers 1134 current unsafe rows`

### `unsafe-ledger-after.jsonl`

- command output from unsafe ledger generation
- result: generated ledger for the post-change source tree
