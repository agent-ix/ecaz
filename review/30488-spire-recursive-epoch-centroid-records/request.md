# Review Request: SPIRE Recursive Epoch Centroid Records

Head SHA: `a6b94336`

## Summary

Recursive routing epoch materialization now preserves the routing draft's
materialized centroid records on `SpireRecursiveRoutingEpochDraft`.

This keeps the parent/child centroid edge payload attached to the same draft
that already contains the epoch manifest, object manifest, placement directory,
root PID, routing objects, and next PID. Durable relation storage and SQL
diagnostics are still follow-up work.

## Files

- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 24 passed.
- `git diff --check`

No PG18 SQL test was run because this is a pure build-draft surface change.

## Review Focus

- Confirm that carrying centroid records on the epoch draft is the right API
  boundary for the upcoming relation persistence step.
- Confirm cloning records from the routing draft is acceptable here, given that
  epoch materialization consumes routing objects but needs to retain the
  centroid edge payload.
