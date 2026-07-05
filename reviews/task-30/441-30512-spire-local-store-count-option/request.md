# Review Request: SPIRE Local Store Count Option

## Checkpoint

- Code commit: `33f7ba1e`
  (`Surface SPIRE local store count option`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local-store count reloption surface

## Summary

This checkpoint adds the bounded `local_store_count` reloption surface without
yet creating auxiliary store relations.

The change:

- adds `local_store_count` to `EcSpireReloptions` and `EcSpireOptions`;
- bounds the value to `1..=16`;
- defaults to `1`, preserving the current embedded single-store path;
- exposes the parsed value through `ec_spire_index_options_snapshot`;
- documents the diagnostic meaning in `docs/SPIRE_DIAGNOSTICS.md`;
- keeps `ambuild` blocked for values above `1` with a clear error until store
  relation creation lands.

This makes the option visible and reviewable while avoiding a misleading state
where `local_store_count = 2` silently builds into the current single relation.

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

- whether `1..=16` is an appropriate first bound for local store count;
- whether blocking executable builds above `1` is the right transitional
  behavior until store relation creation lands;
- whether `ec_spire_index_options_snapshot` is the right place to expose the
  requested store count;
- whether any insert/update/vacuum path needs an additional defensive guard
  before multi-store builds become executable.

## Validation

- `cargo fmt --check`
- `cargo test local_store_count --lib`
- `cargo test default_options_match_phase1_config_contract --lib`
- `git diff --check`
- `git diff --cached --check`

## Notes

No PostgreSQL or PG18 tests were run. This slice changes the SQL diagnostic
shape and Rust reloption parsing but does not create or write auxiliary store
relations.
