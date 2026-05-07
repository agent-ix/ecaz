# Review Request: SPIRE Local Store Tablespace Plan

## Checkpoint

- Code commit: `f27c883b`
  (`Resolve SPIRE local store tablespace plan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local-store tablespace planning

## Summary

This checkpoint resolves the parsed `local_store_tablespaces` reloption into a
descriptor-ready local store tablespace plan.

The change:

- adds `SpireLocalStoreTablespacePlanEntry` with `(local_store_id,
  tablespace_oid)`;
- adds a pure resolver-backed planning helper for deterministic unit coverage;
- preserves repeated tablespace names as repeated OIDs for same-device baseline
  runs;
- defaults omitted `local_store_tablespaces` to the index relation's tablespace;
- uses PostgreSQL `get_tablespace_oid(..., missing_ok = true)` in the build
  path so unknown names fail with an `ec_spire` error before store relation DDL
  lands;
- keeps `local_store_count > 1` non-executable until auxiliary store relations
  are created and opened.

This is a DDL precursor only. It does not create store relations or persist the
store config into root/control metadata.

## Files

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether omitted tablespace names should inherit `pg_class.reltablespace`
  directly at this stage;
- whether resolving names during `ambuild` is the right time before the store
  relation creation slice;
- whether repeated names as repeated OIDs is the correct behavior for baseline
  measurement support;
- whether the plan entry should carry any additional information before DDL
  work starts.

## Validation

- `cargo fmt --check`
- `cargo test local_store_tablespace_plan --lib`
- `cargo pgrx test pg18 test_ec_spire_options_snapshot_sql`
- `git diff --check`
- `git diff --cached --check`

## Notes

The PG18 test was run because the SQL CREATE INDEX path now calls the
tablespace resolver when `local_store_tablespaces = 'pg_default'`.
