# Artifact Manifest: 30750 SPIRE Production Local Statement-Timeout Bridge

Packet: `30750-spire-production-local-statement-timeout-bridge`
Head SHA: `a93088fffb92321ff2c18c9b5e0c88b09489d780`
Timestamp: `2026-05-10T14:16:36Z`

## `cargo-fmt-check.log`

- Command: `script -q -c "cargo fmt --check" review/30750-spire-production-local-statement-timeout-bridge/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -c "cargo check --no-default-features --features pg18" review/30750-spire-production-local-statement-timeout-bridge/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`

## `cargo-pgrx-test-prod-transport-pg.log`

- Command: `script -q -c "cargo pgrx test pg18 prod_transport_pg" review/30750-spire-production-local-statement-timeout-bridge/artifacts/cargo-pgrx-test-prod-transport-pg.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx production transport interrupt and statement-timeout bridge / loopback remote probe using backend query-cancel and timeout indicators / n/a / no heap rerank
- Surface isolation: pg_test loopback transport path; no index table surface.
- Key result: `Discovered 683 SQL entities: 2 schemas, 680 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_prod_transport_pg_interrupt_bridge_cancel ... ok`
- Key result: `test tests::pg_test_ec_spire_prod_transport_pg_statement_timeout_bridge_cancel ... ok`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1564 filtered out; finished in 25.99s`

## `cargo-pgrx-test-local-cancel-regression.log`

- Command: `script -q -c "cargo pgrx test pg18 local_cancel_remote_cancel" review/30750-spire-production-local-statement-timeout-bridge/artifacts/cargo-pgrx-test-local-cancel-regression.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx local-cancel regression / deterministic timer-triggered transport and receive tests / n/a / no heap rerank
- Surface isolation: pg_test loopback transport path; no index table surface.
- Key result: `Discovered 683 SQL entities: 2 schemas, 680 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_prod_transport_local_cancel_remote_cancel ... ok`
- Key result: `test tests::pg_test_ec_spire_prod_receive_local_cancel_remote_cancel ... ok`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1564 filtered out; finished in 28.83s`

## `git-diff-check.log`

- Command: `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30750-spire-production-local-statement-timeout-bridge/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the code/doc checkpoint paths.
