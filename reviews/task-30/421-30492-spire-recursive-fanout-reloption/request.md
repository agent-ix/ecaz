# Review Request: SPIRE Recursive Fanout Reloption

Head SHA: `c095f628`

## Summary

This checkpoint adds an explicit `recursive_fanout` ec_spire reloption.

Contract:

- `recursive_fanout = 0` is the default and preserves current single-level
  `ambuild` behavior.
- `recursive_fanout >= 2` is the opt-in fanout reserved for live recursive build
  activation.
- `recursive_fanout = 1` is rejected by the parsed option accessor because the
  recursive hierarchy builder requires fanout at least 2.

The option is parsed into `EcSpireOptions` but is not wired into live `ambuild`
selection in this checkpoint.

## Files

- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test ec_spire::options -- --nocapture`
  - 7 passed.
- `cargo test recursive_ -- --nocapture`
  - 26 passed.
- `git diff --check`

No PG18 SQL test was run because this slice only adds the parsed opt-in surface;
live SQL behavior remains unchanged.

## Review Focus

- Confirm `recursive_fanout` is the right reloption name and default contract.
- Confirm rejecting `1` in the parsed accessor is acceptable even though the
  local reloption integer bounds allow `0..=max`.
- Confirm deferring live `ambuild` selection to a separate checkpoint keeps the
  behavior transition reviewable.
