# Task 50 Packet 387 Artifact Manifest

- head SHA: `d780f21519405af4c8641f4f283eef0f604f33a5`
- task bucket: `reviews/task-50`
- packet: `reviews/task-50/387-spire-relation-store-tuple-boundary`
- lane: SPIRE unsafe burndown
- fixture/storage/rerank: not applicable; code-level relation store tuple boundary consolidation
- timestamp: `2026-05-21T20:00:07-07:00`
- table surface: not applicable; no benchmark or SQL fixture

## Artifacts

### `cargo-check-pg18-bench.log`

- command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/cargo-check-pg18-bench.log`
- result: passed
- key lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 13.54s`
- note: warning is the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `rustfmt-relation-store.log`

- command: `script -q -c "rustfmt --check src/am/ec_spire/storage/relation_store.rs" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/rustfmt-relation-store.log`
- result: passed
- key lines:
  - rustfmt emitted the existing stable-toolchain warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- command: `script -q -c "git diff --check" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/git-diff-check.log`
- result: passed

### `raw-boundary-guard.log`

- command: `script -q -c "rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/raw-boundary-guard.log`
- result: passed; no matches

### `src-unsafe-count.log`

- command: `script -q -c "rg -n 'unsafe\\s*\\{' src | wc -l" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/src-unsafe-count.log`
- result: `1137`

### `unsafe-ledger-generate.log`

- command: `script -q -c "python3 scripts/unsafe_ledger.py generate --output reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/387-spire-relation-store-tuple-boundary src" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/unsafe-ledger-generate.log`
- result: passed
- key line: `wrote 1137 unsafe ledger rows to reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/unsafe-ledger-check.log`
- result: passed
- key line: `ledger covers 1137 current unsafe rows`

### `unsafe-ledger-after.jsonl`

- command output from unsafe ledger generation
- result: generated ledger for the post-change source tree
