# SPIRE Vacuum Path Complete

## Checkpoint

- Code commit: `ab0fb034`
  (`Mark SPIRE vacuum path complete`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Task-plan closeout for Phase 1 logical delete/vacuum correctness

## Summary

This checkpoint marks the logical delete/vacuum path complete for the Phase 1
single-level foundation.

The already-landed vacuum path covers:

- `ambulkdelete` callback walking over visible base and delta-insert
  assignments
- grouping callback-dead heap locators by base leaf PID
- row-encoded delete-delta object publication
- routed scan suppression by `vec_id`
- `amvacuumcleanup` compaction of active delta objects into replacement V2 base
  leaves
- removal of compacted delta objects from the active placement directory
- no-delta, insert-only, delete-delta, and mixed insert/delete compaction
  coverage
- retired manifest publication before root/control advancement

Physical old-object tuple reclamation and full SQL `VACUUM` end-to-end
coverage remain separate validation/reclamation follow-ups.

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not rerun for this documentation-only closeout. Vacuum coverage was
validated in earlier focused packets and in the latest full SPIRE PG18 lib
suite run:

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `235 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`

## Notes

- This does not implement physical page reclamation for no-longer-active
  object tuples.
- This does not close real SQL `VACUUM` coverage, which remains awkward under
  transactional pgrx pg_tests.
