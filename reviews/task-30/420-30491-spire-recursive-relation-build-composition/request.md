# Review Request: SPIRE Recursive Relation Build Composition

Head SHA: `7443e55c`

## Summary

This checkpoint adds the non-activated relation recursive build composer inside
the build module.

The helper composes the existing Phase 3 pieces:

- train first-level centroids from `SpireBuildState`;
- assemble recursive epoch input from centroid assignments;
- write recursive V2 leaf objects and routing objects through
  `SpireRelationObjectStore`;
- verify the recursive epoch draft next PID agrees with the coordinator cursor;
  and
- publish the relation epoch through the recursive relation publish bridge.

This is not wired into live `ec_spire_ambuild` yet, preserving current
single-level SQL behavior while giving the final recursive SQL smoke a single
call point.

## Files

- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 26 passed.
- `git diff --check`

No PG18 SQL test was run because this helper is compiled but not live-selected
by `ambuild` in this checkpoint.

## Review Focus

- Confirm composing through the existing coordinator/object-store/publish bridge
  is the right final step before live selection.
- Confirm the next-PID agreement check is useful at this boundary.
- Confirm keeping this helper private until live activation avoids exposing a
  premature API surface.
