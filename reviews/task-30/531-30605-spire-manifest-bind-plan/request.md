# SPIRE Manifest Bind Plan

## Summary

This checkpoint adds the manifest-publication counterpart to the remote search
bind-plan surface.

Changes:

- Adds `ec_spire_remote_epoch_manifest_libpq_bind_plan(...)`.
- Expands each manifest dispatch row into the three bind slots defined by
  `ec_spire_remote_epoch_manifest_libpq_parameter_contract()`.
- Reports per-bind parameter ordinal, name, PostgreSQL type, value source,
  value status, preview, and element count.
- Keeps raw conninfo out of the bind surface; the remote index is represented by
  the descriptor-backed remote regclass already used by the request/dispatch
  plan.
- Reports the JSONB manifest payload bind with the payload format and entry
  count instead of dumping the full payload into a preview string.
- Updates the Phase 7 task note with the manifest bind-plan surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `73143f69`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

Result:

- PG18 remote epoch manifest persistence filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`

## Notes

This remains pre-I/O. It prepares the manifest publication executor's bind
slots, but it does not resolve secrets, open libpq connections, send pipeline
requests, or persist remote apply state.
