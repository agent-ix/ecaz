# SPIRE Upgrade State Check Invariant

## Scope

Addresses reviewer feedback from `30637`: the descriptor-state CHECK was
duplicated between fresh bootstrap SQL and the `0.1.0 -> 0.1.1` upgrade SQL.

Code checkpoint: `159556a9` (`Pin SPIRE upgrade descriptor state checks`)

## Changes

- Added PG18 coverage that parses the descriptor-state CHECK state list from
  both `sql/bootstrap.sql` and `ecaz--0.1.0--0.1.1.sql`.
- The test compares both SQL lists against the catalog-state rows from
  `ec_spire_remote_node_descriptor_state_contract()`, so bootstrap, upgrade,
  and Rust contract state drift is caught by one invariant.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_state_upgrade_check_matches_bootstrap`
- `git diff --check`

## Review Focus

- Whether parsing the SQL text in the PG test is sufficient here, or whether
  future migration scripts should expose an explicit generated state-list
  source.
