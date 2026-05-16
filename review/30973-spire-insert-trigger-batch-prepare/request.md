# Review Request: SPIRE INSERT Trigger Batch Prepare

## Summary

Please review commit `c46992c2` (`Batch SPIRE coordinator insert trigger prepares`).

This closes the Phase 12.4 P9 implementation row by moving coordinator-routed
multi-row INSERT from row-at-a-time remote prepare dispatch to a statement-level
batch:

- `ec_spire_enable_coordinator_insert(...)` now installs both a BEFORE ROW queue
  trigger and an AFTER STATEMENT flush trigger.
- The row trigger still extracts the typed payload and plans the owning remote,
  but it queues the tuple payload instead of dispatching immediately.
- The statement trigger drains queued rows in statement order and calls
  `ec_spire_prepare_coordinator_insert_tuple_payload_batch(...)`.
- The Rust batch helper builds all remote tuple-payload INSERT SQL and routes
  the prepares through the Tokio fanout adapter from packet `30972`.
- Descriptor refresh staging dedupes `(node_id, descriptor_generation)` within
  the local statement so same-node rows in one batch do not trip the existing
  monotonic descriptor guard.

The Phase 12 tracker now marks P9 complete, with evidence tied to the batch
queue/flush trigger pair and PG18 trigger fixtures.

## Validation

- `cargo check --no-default-features --features pg18`
  - passes with the existing `src/am/mod.rs` unused-import warning.
- `cargo fmt --check`
  - passes; rustfmt emits existing nightly-only config warnings.
- `git diff --check`
  - passes.
- `cargo pgrx test pg18 test_ec_spire_enable_coordinator_insert_trigger_sql`
  - passed.
- `cargo pgrx test pg18 test_ec_spire_trigger_multirow_commits_prepares_sql`
  - passed.
- `cargo pgrx test pg18 test_ec_spire_coordinator_dml_frontdoor_plan_sql`
  - passed.

## Requested Review

Please focus on:

1. Whether the BEFORE ROW queue plus AFTER STATEMENT flush trigger semantics are
   correct when the BEFORE ROW trigger suppresses local coordinator heap rows.
2. Whether batch flush failure still preserves the no-orphan remote prepared
   transaction contract before local commit/abort callbacks are registered.
3. Whether descriptor refresh dedupe for same-node, same-generation rows is the
   right local-statement behavior.
4. Whether the Phase 12.4 P9 tracker row is now fairly closed.
