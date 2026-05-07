# Review Request: SPIRE Recursive Leaf Input Writer

Head SHA: `b15a48d2`

## Summary

This checkpoint adds a recursive routing epoch input shape that owns leaf
assignment rows instead of requiring callers to prewrite leaf placements.

The new helper:

- validates the recursive routing draft before object writes;
- derives the expected leaf PID -> routing parent PID map from the draft;
- rejects duplicate, unexpected, missing, and parent-drifted leaf inputs;
- writes each leaf as a V2 leaf object through the shared local/relation object
  store writer seam; and
- reuses the existing recursive routing epoch materializer to combine routing
  objects plus newly written leaf placements into a validated snapshot.

## Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo test recursive_routing_epoch_ -- --nocapture`
  - 6 passed, including the new leaf-input write and parent-drift tests.
- `git diff --check`

No PG18 SQL test was run for this slice because it is still a pure build-helper
and object-writer seam checkpoint. Live relation-backed recursive build
orchestration remains open.

## Review Focus

- Confirm the leaf-input coverage checks reject malformed input before partial
  manifest publication can be assembled.
- Confirm the shared object-store seam is the right boundary for the upcoming
  relation-backed recursive build coordinator.
- Check whether this helper should also enforce any row-level invariant now, or
  whether leaf row validation should remain owned by the V2 leaf object writer.
