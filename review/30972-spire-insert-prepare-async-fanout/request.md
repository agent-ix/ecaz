# Review Request: SPIRE INSERT Prepare Async Fanout

## Summary

Please review commit `3f4242e5` (`Add async SPIRE insert prepare fanout`).

This is the first Phase 12.4 P9 slice. It moves
`coordinator_insert_prepare_remote_sql(...)` onto a reusable Tokio fanout
adapter:

- builds one or more remote INSERT prepare requests from dispatch rows;
- opens remote libpq/tokio connections concurrently with the existing
  production transport pattern;
- runs `BEGIN`, remote INSERT SQL, descriptor metadata refresh, and
  `PREPARE TRANSACTION`;
- bridges local query cancel / statement timeout into remote cancel requests;
- rolls back open remote transactions on pre-prepare failure;
- rolls back any successfully prepared remotes if another dispatch in the batch
  fails before local callbacks are registered;
- registers local commit/abort callbacks only after every remote prepare in
  the batch succeeds.

The existing single-row helper now routes through the batch adapter. The
tracker row remains open because the coordinator INSERT trigger still invokes
the helper once per row; statement-level accumulation is the next slice before
we can claim M remote prepares no longer serialize for multi-row INSERT.

## Reviewer Feedback Processed

While this slice was in progress, reviewer feedback arrived for packets `30969`
and `30970`. I processed the actionable P3 notes in this commit:

- added a comment to the placement contention fixture pinning the
  same-transaction app-table plus placement-table write shape;
- added tracker notes clarifying the INSERT cancel fixture's post-prepare
  cleanup coverage and the earlier shared rollback windows.

## Validation

- `cargo check --no-default-features --features pg18`
  - passes with the existing `src/am/mod.rs` unused-import warning.
- `cargo fmt --check`
  - passes; rustfmt emits existing nightly-only config warnings.
- `git diff --check`
  - passes.
- `cargo pgrx test pg18 test_ec_spire_insert_prepare_local_cancel_rolls_back`
  - passed.
- `cargo pgrx test pg18 test_ec_spire_trigger_multirow_commits_prepares_sql`
  - passed.

## Requested Review

Please focus on:

1. Whether the async prepare adapter preserves the no-orphan remote prepared
   transaction contract across partial batch failure and local cancellation.
2. Whether callback registration after all remote prepares succeed is the right
   boundary.
3. Whether keeping the P9 tracker row open until trigger-level batching lands is
   the right scope call.
