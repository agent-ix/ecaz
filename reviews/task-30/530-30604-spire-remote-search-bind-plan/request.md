# SPIRE Remote Search Bind Plan

## Summary

This checkpoint adds a SQL-visible bind-plan surface for the future remote
search libpq executor.

Changes:

- Adds `ec_spire_remote_search_libpq_bind_plan(...)`.
- Expands each remote search dispatch row into the six bind slots defined by
  `ec_spire_remote_search_libpq_parameter_contract()`.
- Reports per-bind parameter ordinal, name, PostgreSQL type, value source,
  value status, preview, and element count.
- Keeps raw conninfo out of the surface; the bind plan uses the registered
  remote index regclass and secret-backed dispatch state already exposed by the
  connection/dispatch plan.
- Reports all bind slots as blocked when the remote descriptor is missing.
- Reports all bind slots as ready when the descriptor-backed dispatch row is
  pipeline-ready.
- Updates the Phase 7 task note with the bind-plan surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `fbe82ab6`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_req_blocked`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `git diff --check`

Result:

- PG18 blocked libpq request filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
- PG18 active descriptor catalog filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`

## Notes

This remains pre-I/O. It prepares the exact bind slots a future executor will
send, but it does not resolve secrets, open libpq connections, enter pipeline
mode, or read remote result rows.
