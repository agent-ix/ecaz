# SPIRE Remote Search Secret Plan

## Scope

Task 30 SPIRE Phase 7 now has an explicit libpq conninfo-secret planning
surface for remote search rows. This keeps external secret lookup visible to
operators and to the future executor without exposing raw conninfo or opening
sockets.

Code checkpoint: `8d904078` (`Add SPIRE remote search secret plan`)

## Changes

- Added `ec_spire_remote_search_libpq_secret_plan(...)`.
- Added `ec_spire_remote_search_libpq_secret_summary(...)`.
- The plan surface derives from the existing dispatch plan, resolves only the
  descriptor secret reference through the external provider policy, and reports
  provider lookup key, resolved byte count, raw-exposure flag, secret action,
  next executor step, status, and recommendation.
- The summary surface aggregates resolved and blocked secret counts plus remote
  PID counts, preserving descriptor blockers ahead of secret lookup when a row
  is not yet dispatch-ready.
- Extended the active remote-node catalog PG18 fixture to set a scoped
  `EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_2` secret and assert that the new
  secret plan advances to `resolved_conninfo` / `open_libpq_connection` while
  the older executor-readiness gate remains at the pre-I/O secret-resolution
  blocker.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `git diff --check`

## Review Focus

- Whether the new secret summary should expose descriptor-blocked counts as a
  separate column, or whether preserving descriptor blocker status/step is
  enough for this executor planning stage.
- Whether a later executor-readiness slice should consume this secret-plan
  status directly, or whether that should wait until the actual libpq executor
  rows are added.
