# Task 50 Packet 385 Artifact Manifest

- head SHA: `e717b6f1eefd4bac870190cba7cb6d39b62fda5e`
- task bucket: `reviews/task-50`
- packet: `reviews/task-50/385-spire-publish-relation-boundary`
- lane: SPIRE unsafe burndown
- fixture/storage/rerank: not applicable; code-level publish relation boundary consolidation
- timestamp: `2026-05-21T19:50:30-07:00`
- table surface: not applicable; no benchmark or SQL fixture

## Artifacts

### `cargo-check-pg18-bench.log`

- command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/385-spire-publish-relation-boundary/artifacts/cargo-check-pg18-bench.log`
- result: passed
- key lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 25.50s`
- note: warning is the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

### `rustfmt-build-publish.log`

- command: `script -q -c "rustfmt --check src/am/ec_spire/build.rs src/am/ec_spire/build/publish.rs" reviews/task-50/385-spire-publish-relation-boundary/artifacts/rustfmt-build-publish.log`
- result: passed
- key lines:
  - rustfmt emitted the existing stable-toolchain warnings for unstable `imports_granularity` and `group_imports`.

### `git-diff-check.log`

- command: `script -q -c "git diff --check" reviews/task-50/385-spire-publish-relation-boundary/artifacts/git-diff-check.log`
- result: passed

### `raw-boundary-guard.log`

- command: `script -q -c "rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src" reviews/task-50/385-spire-publish-relation-boundary/artifacts/raw-boundary-guard.log`
- result: passed; no matches

### `src-unsafe-count.log`

- command: `script -q -c "rg -n 'unsafe\\s*\\{' src | wc -l" reviews/task-50/385-spire-publish-relation-boundary/artifacts/src-unsafe-count.log`
- result: `1149`

### `unsafe-ledger-generate.log`

- command: `script -q -c "python3 scripts/unsafe_ledger.py generate --output reviews/task-50/385-spire-publish-relation-boundary/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/385-spire-publish-relation-boundary src" reviews/task-50/385-spire-publish-relation-boundary/artifacts/unsafe-ledger-generate.log`
- result: passed
- key line: `wrote 1149 unsafe ledger rows to reviews/task-50/385-spire-publish-relation-boundary/artifacts/unsafe-ledger-after.jsonl`

### `unsafe-ledger-check.log`

- command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/385-spire-publish-relation-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/385-spire-publish-relation-boundary/artifacts/unsafe-ledger-check.log`
- result: passed
- key line: `ledger covers 1149 current unsafe rows`

### `unsafe-ledger-after.jsonl`

- command output from unsafe ledger generation
- result: generated ledger for the post-change source tree
