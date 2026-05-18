# Review Request: SPIRE Prepared-Xact Reaper

## Summary

This packet closes Phase 12a.1 by adding coordinator-side prepared-transaction intent metadata and operator-driven recovery for remote prepared xacts stranded in the `PREPARE TRANSACTION` lost-ack window.

Code checkpoint: `35240a79` (`Add SPIRE prepared xact reaper`).

## Changes

- Adds `ec_spire_remote_prepared_xact_intent` to `sql/bootstrap.sql` with `(index_oid, node_id, served_epoch, xid, gid, intent_state)` plus indexes for node/state and index/node/epoch sweeps.
- Records `prepare_requested` before remote prepare dispatch, marks `prepare_acked` after remote prepare success, and marks `commit_local` in a `PreCommit` callback before the existing remote commit callback.
- Adds `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)` and `ec_spire_reap_all_orphaned_remote_prepared_xacts()` operator entrypoints.
- Reaper scans remote `pg_prepared_xacts` for `ec_spire_insert_%`, parses the GID identity, joins coordinator intent metadata, and rolls back entries whose top xid is no longer live and whose intent state is not `commit_local`.
- Documents the named `remote-prepare-lost-ack` failure mode and the v1 operator-driven recovery decision in ADR-069, `docs/SPIRE_LIBPQ_RUNBOOK.md`, and `docs/SPIRE_DIAGNOSTICS.md`.
- Marks Phase 12a.1 complete in `plan/tasks/task30-phase12a-spire-readiness-followups.md`.

## Evidence

See `artifacts/manifest.md`.

- `cargo test prepared_transaction_gid_parser_extracts_reaper_identity --lib`
- `cargo test prepared_transaction_intent_state_validator_matches_catalog_contract --lib`
- `cargo pgrx test pg18 test_ec_spire_reaper_resolves_lost_prepare_ack_fixture`
- `cargo fmt --check`
- `git diff --check`

The PG18 fixture creates the post-WAL-flush lost-ack state directly: a remote prepared transaction exists, coordinator intent remains `prepare_requested`, and recovery uses only `ec_spire_reap_orphaned_remote_prepared_xacts(33)`. The test verifies the prepared xact and its payload row are rolled back.

## Review Focus

1. Confirm the intent-state transitions preserve committed local transactions by marking `commit_local` before remote commit resolution.
2. Confirm the reaper's rollback rule is conservative enough for missing-intent parsed SPIRE GIDs.
3. Confirm the docs no longer overstate manual-only recovery and clearly leave the v1 sweeper operator-driven.
