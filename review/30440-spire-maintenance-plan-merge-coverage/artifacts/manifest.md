# Artifact Manifest: SPIRE Maintenance Plan Merge Coverage

No measurement artifacts.

- head SHA: `67a7e483`
- packet/topic: `30440-spire-maintenance-plan-merge-coverage`
- lane / fixture / storage format / rerank mode: not applicable
- command used: `cargo test maintenance_plan_snapshot --lib`; `cargo fmt --check`; `git diff --check`
- timestamp: 2026-05-04
- isolated one-index-per-table or shared-table surfaces: not applicable
- key result lines: focused unit test filter passed 3 tests; formatting and diff checks passed
