# Review Request: SPIRE Local Store Tablespaces Option

## Checkpoint

- Code commit: `4e48735d`
  (`Surface SPIRE local store tablespaces option`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local-store tablespace reloption surface

## Summary

This checkpoint adds the `local_store_tablespaces` reloption surface for SPIRE
local placement planning without yet creating auxiliary store relations.

The change:

- adds optional `local_store_tablespaces` parsing to `EcSpireReloptions` and
  `EcSpireOptions`;
- trims and normalizes comma-separated tablespace names;
- requires the number of names to match `local_store_count`;
- permits repeated tablespace names so same-device baseline runs can be
  represented honestly;
- exposes the normalized string through `ec_spire_index_options_snapshot`;
- documents the diagnostic meaning in `docs/SPIRE_DIAGNOSTICS.md`;
- keeps `local_store_count > 1` non-executable until auxiliary store relation
  creation lands.

This makes the requested placement surface reviewable while preserving the
current embedded single-store execution path.

## Files

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/scan.rs`
- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether the normalized comma-separated string is sufficient until durable
  store descriptors are wired into relation creation;
- whether repeated tablespace names should remain explicitly allowed for
  same-device baseline runs;
- whether requiring an exact name count match with `local_store_count` is the
  right transitional contract;
- whether `ec_spire_index_options_snapshot` exposes enough state for operators
  to audit intended placement.

## Validation

- `cargo fmt --check`
- `cargo test local_store_tablespaces --lib`
- `cargo test local_store_count --lib`
- `cargo test default_options_match_phase1_config_contract --lib`
- `cargo pgrx test pg18 test_ec_spire_options_snapshot_sql`
- `git diff --check`
- `git diff --cached --check`

## Notes

The focused PG18 snapshot test was run because this slice changes PostgreSQL
reloption parsing and the SQL diagnostics return shape. No PG17 validation was
run.
