# Artifact Manifest: 30747 SPIRE Production State Mode Attribution

Packet: `30747-spire-production-state-mode-attribution`
Head SHA: `81eaec07572c5fa8348d5106fdfa9de90237c6a6`
Timestamp: `2026-05-10T06:21:13-07:00`

## `cargo-fmt-check.log`

- Command: `script -q -c "cargo fmt --check" review/30747-spire-production-state-mode-attribution/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -c "cargo check --no-default-features --features pg18" review/30747-spire-production-state-mode-attribution/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`

## `cargo-test-production-executor.log`

- Command: `script -q -c "cargo test --no-default-features --features pg18 production_executor_ --lib" review/30747-spire-production-state-mode-attribution/artifacts/cargo-test-production-executor.log`
- Lane / fixture / storage format / rerank mode: PG18 Rust production executor state tests plus one dry SQL state-summary test / in-memory executor fixtures and one isolated RaBitQ dry index / RaBitQ / no heap rerank
- Surface isolation: includes one isolated one-index-per-table dry SQL surface; Rust state fixtures are in-memory.
- Key result: `test am::ec_spire::production_executor_state_tests::production_executor_degraded_missing_secret_skips_receive_request ... ok`
- Key result: `test am::ec_spire::tests::production_executor_state_keeps_admitted_dispatches_dry ... ok`
- Key result: `test tests::pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
- Key result: `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1542 filtered out; finished in 13.92s`

## `cargo-pgrx-test-prod-executor-session-policy.log`

- Command: `script -q -c "cargo pgrx test pg18 prod_executor_session_policy" review/30747-spire-production-state-mode-attribution/artifacts/cargo-pgrx-test-prod-executor-session-policy.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx session-policy test / production executor dry fixture / RaBitQ / no heap rerank
- Surface isolation: isolated one-index-per-table test surface.
- Key result: `test tests::pg_test_ec_spire_prod_executor_session_policy_guc ... ok`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1562 filtered out; finished in 24.33s`

## `git-diff-check.log`

- Command: `script -q -c "git diff --check -- src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30747-spire-production-state-mode-attribution/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the code/doc checkpoint paths.
