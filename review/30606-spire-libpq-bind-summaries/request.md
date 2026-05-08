# SPIRE Libpq Bind Summaries

## Summary

This checkpoint adds compact readiness summaries for the search and manifest
libpq bind-plan surfaces.

Changes:

- Adds `ec_spire_remote_search_libpq_bind_summary(...)`.
- Aggregates remote search bind rows into request count, bind count,
  ready/blocked bind counts, remote PID counts, blocked PID counts, and
  effective bind status.
- Adds `ec_spire_remote_epoch_manifest_libpq_bind_summary(...)`.
- Aggregates manifest bind rows into request count, bind count, ready/blocked
  bind counts, parameter count, manifest entry count, executor status, and
  effective bind status.
- Updates active, blocked, and manifest publication PG18 coverage.
- Updates the Phase 7 task note with both summary surfaces.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `aad38b51`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_req_blocked`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

Result:

- PG18 blocked libpq request filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
- PG18 active descriptor catalog filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
- PG18 remote epoch manifest persistence filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`

## Notes

This remains pre-I/O. The summaries give the future executor and operator
diagnostics a compact gate for bind readiness, but they do not resolve secrets,
open libpq connections, or send pipeline messages.
