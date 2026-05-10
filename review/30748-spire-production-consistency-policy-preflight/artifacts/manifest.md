# Artifact Manifest: 30748 SPIRE Production Consistency Policy Preflight

Packet: `30748-spire-production-consistency-policy-preflight`
Head SHA: `ba10fb410640fb9f2ca20a9fe8c4517b1ea420ff`
Timestamp: `2026-05-10T06:35:17-07:00`

## `cargo-fmt-check.log`

- Command: `script -q -c "cargo fmt --check" review/30748-spire-production-consistency-policy-preflight/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode: static formatting / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; only known stable-rustfmt warnings were emitted.

## `cargo-check-pg18.log`

- Command: `script -q -c "cargo check --no-default-features --features pg18" review/30748-spire-production-consistency-policy-preflight/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode: PG18 compile check / none / n/a / n/a
- Surface isolation: n/a
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.12s`

## `cargo-pgrx-test-prod-consistency-policy.log`

- Command: `script -q -c "cargo pgrx test pg18 prod_consistency_policy_summary_mode_mismatch" review/30748-spire-production-consistency-policy-preflight/artifacts/cargo-pgrx-test-prod-consistency-policy.log`
- Lane / fixture / storage format / rerank mode: PG18 pgrx consistency-policy preflight / isolated one-index-per-table SPIRE fixture / RaBitQ-compatible default index / no heap rerank
- Surface isolation: isolated one-index-per-table test surface.
- Key result: `Discovered 681 SQL entities: 2 schemas, 678 functions, 0 types, 0 enums, 1 sqls, 0 ords, 0 hashes, 0 aggregates, 0 triggers`
- Key result: `test tests::pg_test_ec_spire_prod_consistency_policy_summary_mode_mismatch ... ok`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1563 filtered out; finished in 24.13s`

## `git-diff-check.log`

- Command: `script -q -c "git diff --check -- src/am/mod.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30748-spire-production-consistency-policy-preflight/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode: static whitespace check / none / n/a / n/a
- Surface isolation: n/a
- Key result: exit 0; no whitespace errors in the code/doc checkpoint paths.
