# SPIRE Conninfo Secret Status Surface

## Scope

Task 30 SPIRE Phase 7 now has the first executor-owned secret-provider lookup
surface for `conninfo_secret_name` references.

Code checkpoint: `0a11b293` (`Add SPIRE conninfo secret status surface`)

## Changes

- Added `ec_spire_remote_conninfo_secret_resolution_status(conninfo_secret_name)`.
- The v1 provider maps a secret reference to an external environment key:
  `EC_SPIRE_REMOTE_CONNINFO_` plus the uppercased reference with non-alphanumeric
  bytes replaced by `_`.
- The SQL surface reports provider policy, secret reference, provider lookup
  key, resolved byte count, raw-exposure boolean, status, and recommendation.
- The raw libpq conninfo string is read internally only to determine nonempty
  resolution and byte count; it is not returned through SQL.
- Added PG18 coverage for missing and resolved provider entries, including the
  no-raw-conninfo exposure invariant.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_conninfo_secret_resolution_status`
- `git diff --check`

## Review Focus

- Whether environment-key lookup is an acceptable first external provider for
  the libpq executor slice, or whether the next slice should immediately add a
  provider abstraction behind this status function.
- Whether reporting the provider lookup key is acceptable while continuing to
  suppress raw conninfo.
- Whether the real executor should consume this status surface directly or call
  a private resolver that returns raw conninfo only inside the executor.
