# Task 43 Review Request: SPIRE Vacuum Delete-Delta Coverage

## Summary

This packet closes the SPIRE vacuum/delete-delta visibility gap in the Task 43
tracker.

Changes:

- Adds `miri_collect_visible_assignments_excludes_delete_delta_targets`, which
  builds a real local object-store snapshot containing a base leaf plus a
  delete/insert delta and proves vacuum visibility excludes the deleted base
  vec-id and boundary replica while keeping the live base row and delta insert.
- Promotes existing delta snapshot tests to `miri_` for delete-delta
  publication and rejection of unknown, mismatched, stale, duplicate, and
  already-deleted delete targets.
- Promotes the replacement fold test that proves active deltas are folded into
  replacement leaf rows with deleted rows excluded.

This is not a final completion packet. Mutation probes, SPIRE careful mirror
work/blockers, final aggregate lanes, and final audit remain open.

## Code Under Review

Code commit: `cc79787911a7aec2080c49af34e91ef4700c0af7`

Changed files:

- `src/am/ec_spire/vacuum/tests.rs`
- `src/am/ec_spire/update/tests/delta_snapshot.rs`

## Validation

Artifacts are packet-local under `artifacts/`; see
`artifacts/manifest.md` for commands and key result lines.

- `miri-spire-vacuum-visible-delete-delta.log`: 1 passed; 0 failed.
- `miri-spire-delta-snapshot.log`: 6 passed; 0 failed.
- `miri-spire-delta-replacement-fold.log`: 1 passed; 0 failed.
- `cargo-fmt-check.log`: exit 0.
- `git-diff-check.log`: exit 0.

## Tracker Update

`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
has been updated to mark G5 and the SPIRE vacuum/delete-delta matrix rows
done, with cargo-careful mirroring kept blocked on pgrx/SPIRE harness work.
