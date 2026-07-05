# Review Request: SPIRE Recursive Fanout Validation

Head SHA: `75c158e7`

## Summary

`recursive_fanout = 1` now fails during relation option parsing, so a SPIRE
index cannot be created with the meaningless disabled/enabled sentinel value.
The existing accessor still validates defensively before converting to
`Option<u32>`.

The reloption help text now states the user-facing contract directly:
`0` keeps single-level behavior and recursive fanout values must be at least
`2`.

## Files

- `src/am/ec_spire/options.rs`
- `src/lib.rs`

## Validation

- `cargo test recursive_fanout_validation_rejects_one -- --nocapture`
  - 1 passed: `recursive_fanout_validation_rejects_one`.
- `cargo test recursive_fanout_one_rejected -- --nocapture`
  - 1 passed: `pg_test_ec_spire_recursive_fanout_one_rejected`.
  - PostgreSQL raised
    `ec_spire recursive_fanout reloption must be 0 or at least 2` during
    `CREATE INDEX ... WITH (recursive_fanout = 1)`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm `relation_options` is the right parse-time barrier for rejecting
  `recursive_fanout = 1`.
- Confirm the accessor-level validation is useful defensive coverage rather
  than unnecessary duplication.
- Confirm the reloption help text is clear enough for SQL users.
