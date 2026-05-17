# Review Request: SPIRE 12c Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `87be233efc4469d2756e4ee1d742c453012f1c4a`
- task rows: `12c.1.c`, `12c.5.b`, `12c.11.a`

## Summary

Reconciles three split-tracker rows with tests and invariants already present
on the branch. This is a tracker-only checkpoint: no test or production code
changed in this slice.

## Evidence

- `12c.1.c`
  - Evidence: `src/am/ec_spire/custom_scan/tests.rs`
    `custom_scan_recheck_returns_true_for_epq_stale_row_contract`.
  - The same block also asserts the installed CustomScan methods keep
    `MarkPosCustomScan` and `RestrPosCustomScan` unset, but this packet only
    flips the narrower recheck row.
- `12c.5.b`
  - Evidence: `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`
    `prepared_transaction_intent_transitions_cannot_bypass_prepare_ack`.
  - Evidence: `src/am/ec_spire/coordinator/remote_candidates/resolve.rs`
    has the `#[cfg(test)]` transition invariant inside
    `coordinator_prepared_xact_intent_mark`.
- `12c.11.a`
  - Evidence: `src/tests/remote_search/catalog_cleanup_policy.rs`
    `test_ec_spire_remote_pk_select_isolation_contract_sql`.
  - The current matrix includes `READ COMMITTED`, `REPEATABLE READ`, and
    `SERIALIZABLE`, and asserts the expected distributed PK SELECT title after
    a concurrent remote update for each isolation level.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 prepared_transaction_intent_transitions_cannot_bypass_prepare_ack --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.

## Review Focus

- Confirm the tracker flips are scoped to rows with concrete current-code
  evidence.
- Confirm leaving adjacent stale-looking rows unchecked is correct until their
  evidence is separately audited or extended.
