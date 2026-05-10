# Review Request: SPIRE Production Receive Epoch And Empty Coverage

## Summary

Code checkpoint: `bf539969c285e2080f711b0ed7cacc4a53bc9a89`

This slice closes two production compact-candidate receive edge cases and folds
the latest reviewer P2 planning into Phase 11:

- Production receive now preserves stale served-epoch validation as the
  operator-visible `served_epoch_mismatch` category instead of collapsing it to
  generic candidate batch validation failure.
- `top_k = 0` compact receive returns a ready empty candidate batch rather than
  a failure or missing batch.
- PG18 production receive coverage now proves nonempty loopback receive,
  top-k-zero ready-empty behavior, and stale served-epoch rejection in the same
  focused `prod_receive` lane.
- Phase 11 task/design docs now pin the C5 AM-boundary follow-ons from review:
  the real PostgreSQL interrupt bridge into the cancel token, per-query
  strict/degraded mode threading, and the strict/degraded fault matrix.

This is still pre-C5. The tests exercise the production receive adapter and
state helper, not final SQL row delivery through the AM scan path.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
  - validation failure category mapping for stale served epochs
- `src/lib.rs`
  - `test_ec_spire_prod_receive_top_k_zero`
  - `test_ec_spire_prod_receive_stale_epoch`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 prod_receive`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is preserving `served_epoch_mismatch` through the production validation
  failure category the right operator taxonomy, or should validation return a
  typed enum before C5?
- Is `top_k = 0` as a ready empty batch the right handoff contract for compact
  merge and Stage D heap resolution?
- Do the C5 task/design additions capture the reviewer-requested production
  gates for PostgreSQL cancellation and strict/degraded AM-boundary policy?
