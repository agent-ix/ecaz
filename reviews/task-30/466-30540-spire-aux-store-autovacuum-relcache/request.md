# Review Request: SPIRE Aux Store Autovacuum Relcache Guard

- Code commit: `76aad7fc` (`Cover SPIRE aux store autovacuum relcache`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement coverage
- Agent: coder1

## Summary

This checkpoint closes the important Phase 4 autovacuum behavior concern with a
deterministic PG18 guard instead of a timing-dependent autovacuum launcher test.

The new test `test_ec_spire_aux_store_relcache_disables_autovacuum` builds a
two-store relation-backed SPIRE index, opens the created auxiliary heap store
relations through PostgreSQL relcache, and asserts the parsed
`StdRdOptions.autovacuum.enabled` flag is false for each auxiliary relation.
That proves the `autovacuum_enabled=false` reloption is visible at the boundary
PostgreSQL autovacuum consults, not just stored as raw `pg_class.reloptions`
text.

This checkpoint also cleans stale Phase 4 tracker wording around local-store
planning, relation creation/publication, and implementation status. The status
row now records the aux-store relcache guard and moves T30 to 94%.

## Review Focus

1. Confirm the relcache assertion is the right deterministic boundary for
   Phase 4 autovacuum behavior coverage.
2. Check that the auxiliary relation lookup and relation-open/close handling
   are appropriate for PG18 `pg_test` isolation.
3. Verify the wording cleanup no longer implies auxiliary store DDL/build
   publication are still deferred.
4. Confirm this packet does not overclaim actual autovacuum launcher execution
   or production multi-NVMe performance behavior.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo pgrx test pg18 test_ec_spire_aux_store_relcache_disables_autovacuum`

PG17 was not run; this is a PG18 Phase 4 relation-backed local-store behavior
guard.

## Notes

Full multi-store REINDEX lifecycle support remains intentionally reserved for a
later phase. This packet keeps the Phase 4 scope to local placement behavior and
the parsed PostgreSQL reloption boundary used by autovacuum.
