# Review Request: SPIRE Recursive SQL Parity Breadth

Head SHA: `632d3346`

## Summary

The flat-vs-recursive SQL parity test now checks more than one `LIMIT 1`
query. It keeps the same four-row fixture but compares:

- four query vectors;
- both positive and negative routing sides;
- `LIMIT 2` ID-set equality via `array_agg(id ORDER BY id)`; and
- `LIMIT 1` parity for each query.

The expected top-2 sets are asserted for the flat index first, then the
recursive index must match the flat result.

## Files

- `src/lib.rs`

## Validation

- `cargo test flat_recursive_same_candidate -- --nocapture`
  - 1 passed: `pg_test_ec_spire_flat_recursive_same_candidate`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm this is enough SQL parity breadth for the Phase 3 closeout without
  turning the smoke into a recall benchmark.
- Confirm `LIMIT 2` ID-set equality is the right assertion for this fixture.
- Confirm the positive and negative query vectors exercise both recursive root
  branches.
