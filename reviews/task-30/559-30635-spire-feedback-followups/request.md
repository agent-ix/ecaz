# SPIRE Feedback Follow-ups

## Scope

This packet closes the concrete follow-ups called out in reviewer feedback on
30627, 30628, and 30630.

Code checkpoints:

- `5e2220fa` (`Update SPIRE cleanup operator docs`)
- `35e2d229` (`Test SPIRE remote operator entrypoints`)

## Changes

- Updated `docs/SPIRE_DIAGNOSTICS.md` to include
  `ec_spire_index_epoch_cleanup_run(index_oid)` in the starter list and
  function map.
- Updated the cleanup status taxonomy in the docs to include
  `not_required`, `blocked_by_retention`, and `supported`.
- Documented that `ec_spire_index_epoch_cleanup_run(index_oid)` holds the
  SPIRE publish lock and should be scheduled by an operator-controlled job
  during an acceptable publish-path pause window.
- Extended `test_ec_spire_remote_phase7_policy_contracts` so every function
  named by `ec_spire_remote_operator_entrypoint_contract()` must exist in
  `pg_proc`.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Review Focus

- Whether the operator cleanup docs now accurately reflect the post-30628
  physical cleanup lifecycle.
- Whether `pg_proc` existence is a sufficient invariant for the remote operator
  entrypoint contract, or whether later work should strengthen this to
  signature-level reachability.
