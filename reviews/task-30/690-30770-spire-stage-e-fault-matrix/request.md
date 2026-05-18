# 30770 - SPIRE Stage E Fault Matrix

## Summary

This packet reviews commit `c5758c2541d8262276c81c52eb28ae6cae2995bf`
(`Expose SPIRE Stage E fault matrix`).

The slice prepares Phase 11 Stage E before implementing the one-coordinator /
two-remote local fixture.

Changes:

- Adds `ec_spire_remote_search_stage_e_fault_matrix()`, a SQL-visible matrix
  for the Stage E fixture cases.
- Covers epoch mismatch, version skew, endpoint fingerprint mismatch,
  connection reset mid-batch, remote backend termination, remote statement
  timeout, local statement timeout, local cancel, simulated network partition,
  remote OOM, and missing/reindexed remote index.
- Each fixture case states the production failure category, next executor
  step, strict action/status, degraded action/status, expected counter delta,
  and required evidence.
- Registers the new Stage E matrix in the operator entrypoint contract and
  updates the reachable entrypoint count.
- Adds a production fault-matrix row for
  `requires_remote_row_materialization`, folding missing or stale coordinator
  materialization mappings under the existing ADR-064 blocker. This addresses
  the `30765` row-materialization mapping matrix follow-up without inventing
  premature provider runtime categories.
- Updates Phase 11 to mark the pre-fixture fault contract complete.

## Key Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo test stage_e_fault_matrix --no-default-features --features pg18`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo pgrx test pg18 test_ec_spire_stage_e_fault_matrix_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`

All commands passed.

## Review Focus

- Is the Stage E fixture matrix complete enough to implement the local
  one-coordinator / two-remote strict/degraded fault harness without adding
  policy decisions during fixture work?
- Are failure categories mapped to the right production executor steps?
- Is `requires_remote_row_materialization` the right production matrix category
  for missing or stale coordinator materialization mappings until the provider
  implementation lands?
- Does the operator entrypoint contract correctly expose this as a fixture
  contract rather than claiming the fixture is implemented?
