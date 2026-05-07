# Review Request: SPIRE Recursive Options Diagnostics

Head SHA: `ec5bbf8b`

## Summary

`ec_spire_index_options_snapshot(index_oid)` now exposes recursive build
configuration directly:

- `recursive_fanout`
- `recursive_build_enabled`

The options and scan-sanity diagnostics now count active leaves through the
recursive hierarchy when `recursive_fanout >= 2`, instead of treating root
children as leaves. That keeps recursive indexes from reporting the root's
internal-routing children as the active leaf count.

## Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test options_snapshot_sql -- --nocapture`
  - 1 passed, including PG18 pg-test
    `pg_test_ec_spire_options_snapshot_sql`.
- `cargo fmt`
- `git diff --check`

## Review Focus

- Confirm `recursive_fanout` and `recursive_build_enabled` are the right
  SQL-visible build-option fields.
- Confirm options and scan-sanity diagnostics should use recursive leaf counts
  whenever recursive build is enabled.
- Confirm leaving per-level `nprobe` metadata explicitly deferred is still the
  right Phase 3 boundary.
