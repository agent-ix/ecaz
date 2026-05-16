# Artifact Manifest: 30751 SPIRE Production Strict/Degraded Fault Matrix

Packet: `30751-spire-production-strict-degraded-fault-matrix`
Head SHA: `0cbf841cb2f5e678d1bb7df4568cdc7ca8010eae`
Timestamp: `2026-05-10T14:32:51Z`

## `cargo-fmt-check.log`

- Command: `script -q -c "cargo fmt --check" review/30751-spire-production-strict-degraded-fault-matrix/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -c "cargo check --no-default-features --features pg18" review/30751-spire-production-strict-degraded-fault-matrix/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.18s`

## `cargo-test-fault-matrix-coverage.log`

- Command: `script -q -c "cargo test production_fault_matrix_covers_required_categories --no-default-features --features pg18" review/30751-spire-production-strict-degraded-fault-matrix/artifacts/cargo-test-fault-matrix-coverage.log`
- Lane / fixture / storage format / rerank mode: Rust unit contract coverage / static matrix / n/a / n/a
- Surface isolation: n/a
- Key result: `test am::ec_spire::production_executor_state_tests::production_fault_matrix_covers_required_categories ... ok`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1567 filtered out; finished in 0.00s`

## `cargo-pgrx-test-production-fault-matrix.log`

- Command: `script -q -c "cargo pgrx test pg18 production_fault_matrix_contract" review/30751-spire-production-strict-degraded-fault-matrix/artifacts/cargo-pgrx-test-production-fault-matrix.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx SQL-visible contract / dry fault-matrix table / n/a / n/a
- Surface isolation: dry static table; no index or remote socket surface.
- Key result: `Discovered 685 SQL entities: 2 schemas, 682 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_production_fault_matrix_contract ... ok`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1567 filtered out; finished in 25.87s`

## `git-diff-check.log`

- Command: `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/types.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30751-spire-production-strict-degraded-fault-matrix/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the code/doc checkpoint paths.
