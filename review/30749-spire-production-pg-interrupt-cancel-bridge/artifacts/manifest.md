# Artifact Manifest: 30749 SPIRE Production PG Interrupt Cancel Bridge

Packet: `30749-spire-production-pg-interrupt-cancel-bridge`
Head SHA: `7af6fb0d32aca62d4f5062b806d87896920f02b1`
Timestamp: `2026-05-10T13:59:00Z`

## `cargo-fmt-check.log`

- Command: `script -q -c "cargo fmt --check" review/30749-spire-production-pg-interrupt-cancel-bridge/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -c "cargo check --no-default-features --features pg18" review/30749-spire-production-pg-interrupt-cancel-bridge/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`

## `cargo-pgrx-test-prod-transport-pg-interrupt.log`

- Command: `script -q -c "cargo pgrx test pg18 prod_transport_pg_interrupt_bridge_cancel" review/30749-spire-production-pg-interrupt-cancel-bridge/artifacts/cargo-pgrx-test-prod-transport-pg-interrupt.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx production transport interrupt bridge / loopback remote probe using backend query-cancel flags / n/a / no heap rerank
- Surface isolation: pg_test loopback transport path; no index table surface.
- Key result: `Discovered 682 SQL entities: 2 schemas, 679 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_prod_transport_pg_interrupt_bridge_cancel ... ok`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1564 filtered out; finished in 24.67s`

## `cargo-pgrx-test-local-cancel-regression.log`

- Command: `script -q -c "cargo pgrx test pg18 local_cancel_remote_cancel" review/30749-spire-production-pg-interrupt-cancel-bridge/artifacts/cargo-pgrx-test-local-cancel-regression.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx local-cancel regression / deterministic timer-triggered transport and receive tests / n/a / no heap rerank
- Surface isolation: pg_test loopback transport path; no index table surface.
- Key result: `Discovered 682 SQL entities: 2 schemas, 679 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_prod_transport_local_cancel_remote_cancel ... ok`
- Key result: `test tests::pg_test_ec_spire_prod_receive_local_cancel_remote_cancel ... ok`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1563 filtered out; finished in 25.95s`

## `git-diff-check.log`

- Command: `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30749-spire-production-pg-interrupt-cancel-bridge/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the code/doc checkpoint paths.
