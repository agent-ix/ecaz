# Manifest: SPIRE CustomScan Root-Control Wrapper

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/393-spire-customscan-root-control-wrapper`
- Code commit: `441ee67541de99f5570c8ae182a772ec1a5abeb0`
- Branch: `task-50-unsafe-closeout`
- Scope:
  - `src/am/ec_spire/custom_scan/planner.rs`
  - `src/am/ec_spire/coordinator/snapshots.rs`
- Plan program: P2 PostgreSQL Handle Views
- Count movement: `1124` -> `1123`

## Artifacts

- `rustfmt-check.log`
  - Command: `rustfmt --check src/am/ec_spire/coordinator/snapshots.rs src/am/ec_spire/custom_scan/planner.rs`
  - Result: passed with existing stable-toolchain warnings for unstable
    rustfmt settings.

- `cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed.
  - Warning: existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

- `git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.

- `raw-boundary-guard.log`
  - Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - Result: no matches.

- `src-unsafe-count.log`
  - Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
  - Result: `1123`.

- `unsafe-ledger-after.jsonl`
  - Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/393-spire-customscan-root-control-wrapper/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/393-spire-customscan-root-control-wrapper src`
  - Result: generated `1123` current unsafe ledger rows.

- `unsafe-ledger-generate.log`
  - Command log for ledger generation.

- `unsafe-ledger-check.log`
  - Command: `python3 scripts/unsafe_ledger.py check --ledger reviews/task-50/393-spire-customscan-root-control-wrapper/artifacts/unsafe-ledger-after.jsonl src`
  - Result: `ledger covers 1123 current unsafe rows`.
