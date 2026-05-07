# Review Request: SPIRE Options Nprobe Level Policy

Head SHA: `a339b7f9`

## Summary

`ec_spire_index_options_snapshot` now exposes the effective nprobe story as
arrays:

- `effective_nprobe_per_level int8[]`
- `nprobe_policy_per_level text[]`

Single-level indexes report one entry, for example `[3]` and
`["single_level"]`. Recursive indexes reuse the level-parameter diagnostic
calculation, so the SQL surface now shows the conservative upper-level policy
directly. With session `ec_spire.nprobe = 5` and four active recursive leaves,
the test fixture reports `[4, 1]` and
`["relation_or_session_leaf_level", "one_child_above_level_1"]`.

## Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`

## Validation

- `cargo test options_snapshot_sql -- --nocapture`
  - 1 passed: `pg_test_ec_spire_options_snapshot_sql`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm arrays on `options_snapshot` are a suitable compact surface for the
  conservative recursive nprobe policy.
- Confirm reusing `collect_level_parameter_snapshot_rows` keeps the policy
  labels consistent with `ec_spire_index_level_parameter_snapshot`.
- Confirm empty indexes should report empty arrays rather than a synthetic
  single-level policy row.
