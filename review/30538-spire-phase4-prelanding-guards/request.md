# Review Request: SPIRE Phase 4 Pre-Landing Guards

- Code commit: `790abba0` (`Reject SPIRE multistore reindex`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement final cleanup
- Agent: coder1

## Summary

This checkpoint addresses the two concrete pre-landing asks from the final
Phase 4 review:

1. Multi-store SPIRE `REINDEX` now rejects explicitly.
2. `SpireRelationObjectStoreSet` constructors now clean up partially opened
   auxiliary store relations if construction returns early.

For `REINDEX`, Phase 4 does not yet own the full auxiliary-store rebuild and
retirement lifecycle. The build path now reports a clear unsupported-lifecycle
error when it sees the existing `ec_spire_store_<index_oid>_<store_id>`
relation that a multi-store REINDEX would need to replace. Single-store
REINDEX remains allowed.

For constructor cleanup, both relation-store-set constructors now use an
`OpenedRelationsGuard` while they are still assembling `Self`. If opening a
later store relation fails, the guard closes earlier opened relations in
reverse order. On success, `into_inner()` moves the relations into
`SpireRelationObjectStoreSet`, whose existing `Drop` continues to own normal
scope-exit cleanup.

The tracker/design notes now record that Phase 4 rejects multi-store REINDEX
until the full lifecycle lands, and the T30 status moves to 92%.

## Review Focus

1. Confirm the multi-store REINDEX rejection point is acceptable for Phase 4:
   explicit, tested, and only triggered when auxiliary local-store relations
   already exist.
2. Confirm single-store REINDEX still exercises the original index-relation
   path and is not blocked by the new message.
3. Verify `OpenedRelationsGuard::into_inner()` prevents double-closing on the
   success path while still closing partially opened relations on early return.
4. Check that the task/design wording distinguishes the current explicit
   rejection from a future full multi-store REINDEX lifecycle.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo pgrx test pg18 test_ec_spire_multistore_reindex_rejected`
- `cargo pgrx test pg18 test_ec_spire_singlestore_reindex_succeeds`

PG17 was not run; this is a PG18 Phase 4 relation-backed multi-store cleanup
slice.

## Notes

The broader checklist in `review/30509-spire-phase4-local-placement-design/
feedback/2026-05-06-05-reviewer.md` contains larger structural and quality
items (`include!` to `mod`, unsafe comments, rustdoc, typed errors, larger
fixtures, codec property tests, autovacuum behavioral testing). Those are
materially larger than a final pre-landing guard patch and should be scheduled
as follow-on hardening packets unless the landing bar is intentionally widened
from Phase 4 local-placement correctness to subsystem-wide cleanup.
