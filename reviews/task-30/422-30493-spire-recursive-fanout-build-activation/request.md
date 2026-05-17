# Review Request: SPIRE Recursive Fanout Build Activation

Head SHA: `670cc55f`

## Summary

This checkpoint activates live recursive SPIRE builds behind the explicit
`recursive_fanout` reloption.

Behavior:

- default `recursive_fanout = 0` still uses the existing single-level relation
  build path;
- `recursive_fanout >= 2` uses the recursive relation build composer;
- recursive builds publish root/internal/leaf hierarchy metadata; and
- the existing recursive scan path can query the published recursive hierarchy.

The new PG18 smoke builds four vectors with `nlists = 4, recursive_fanout = 2`,
verifies hierarchy diagnostics report two internal routing objects and depth 2,
verifies root-routing diagnostics expose the root's two internal children, and
executes an ordered query through the resulting index.

## Files

- `src/am/ec_spire/build.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test ec_spire::options -- --nocapture`
  - 7 passed.
- `cargo test recursive_ -- --nocapture`
  - 27 passed, including PG18 pg-test
    `pg_test_ec_spire_recursive_fanout_build_hierarchy`.
- `git diff --check`

## Review Focus

- Confirm the live selection rule keeps default single-level build behavior
  unchanged.
- Confirm the PG smoke assertions cover the minimum relation-backed recursive
  build contract before the final flat-vs-recursive comparison packet.
- Confirm ordered scan through the recursive hierarchy is acceptable as the first
  activation proof, even though per-level `nprobe` metadata and centroid SQL
  diagnostics remain follow-up items.
