# Review Request: SPIRE INSERT Prepare Local Cancellation

Code checkpoint: `0283ee7baa3ef898c1c8665e27fb8774778f8440` (`Cancel SPIRE insert prepare on local interrupt`)

## Scope

- Adds a cancellation watcher around `coordinator_insert_prepare_remote_sql`,
  the synchronous libpq path used by coordinator-routed INSERT 2PC prepare.
- The watcher polls local PostgreSQL `InterruptPending` /
  `QueryCancelPending`, sends the synchronous libpq cancel token to the remote
  query, and classifies the result as local cancellation.
- On local cancellation, the INSERT prepare path rolls back open remote
  transactions. If cancellation is observed immediately after `PREPARE
  TRANSACTION`, it explicitly rolls back that prepared transaction before
  registering local commit/abort callbacks.
- Adds PG18 loopback fixture
  `test_ec_spire_insert_prepare_local_cancel_rolls_back`: the remote SQL
  cancels the coordinator backend while the remote prepare SQL is sleeping, and
  the fixture asserts `local_query_cancelled`, no matching remote
  `pg_prepared_xacts`, and no visible remote row.
- Closes the Phase 12.4 INSERT 2PC dispatch cancellation tracker row.

## Validation

- `git diff --check 0283ee7b^ 0283ee7b`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_insert_prepare_local_cancel_rolls_back`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and result lines.

## Review Focus

- Confirm the sync libpq watcher is an acceptable parity mechanism with the
  Stage C read-path cancellation bridge.
- Confirm all cancellation windows in INSERT prepare clean up remote state:
  remote SQL failure, descriptor metadata failure, `PREPARE TRANSACTION`
  failure, and cancellation observed immediately after prepare success.
- Confirm the loopback fixture proves the no-orphan prepared-transaction
  contract without relying on shell fixture scripts.
