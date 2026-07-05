# Review Request: SPIRE Writer Identity Contract Status

## Summary

The SQL-visible vector identity contract now includes writer-side Phase 11.2
status rows for the global-ID path.

Code checkpoint: `963044293a1bdda545e4dbacaaa528e0b64fd5e5`
(`Expose SPIRE writer identity contract status`)

## Scope

- Extends `ec_spire_remote_search_vector_identity_contract()` with writer-side
  rows for:
  - the landed `SpireVecIdSourceIdentity` allocation hook;
  - the requirement for stable source identity rather than heap TID;
  - the current Leaf V2 base-object local-ID storage blocker;
  - row-encoded delta assignment support for global IDs.
- Updates the PG contract test to assert the new rows and count.
- Updates the Phase 11 task file to record the SQL-visible contract checkpoint.

## Validation

- `cargo fmt --check`
- `cargo test remote_search_final_contract --lib`
  - 1 passed; 0 failed; 1499 filtered out
- `git diff --check`

## Notes

This is still contract/status work, not full writer global-ID emission. The
next implementation choices are the stable source-identity input surface and
the Leaf V2 variable-width/global vec-id storage change.
