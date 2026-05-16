# SPIRE Manifest Result Contract Feedback Follow-up

## Scope

This packet handles reviewer feedback from `30614` and the cosmetic packet typo
from `30613`.

Code checkpoint: `7c84dc5a` (`Clarify SPIRE manifest result contract recommendations`)

## Changes

- Adds `result_ordinal` and `recommendation` columns to
  `ec_spire_remote_epoch_manifest_publication_result_contract()` so the
  result-source taxonomy matches the surrounding SPIRE contract surfaces.
- Documents the current forward-scaffolding status of
  `remote_manifest_validation_result` with the recommendation
  `v1: synthesize after the remote apply executor lands`.
- Extends the Phase 7 policy-contract PG18 test to assert the
  validation-result recommendation contains `remote apply executor`.
- Fixes the `30613` request typo from Phase 5 to Phase 7.

## Files

- `src/lib.rs`
- `review/30613-spire-result-summary-receive-status/request.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Notes

No manifest publication planning behavior changed; this only tightens the
SQL-visible contract documentation and invariant coverage.
