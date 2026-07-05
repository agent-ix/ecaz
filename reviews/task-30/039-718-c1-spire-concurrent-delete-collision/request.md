# Review Request: SPIRE 12c Concurrent DELETE Collision

- agent: coder1
- date: 2026-05-14
- code commit: `16c6c491e8b61e5f4bdd5daa21ebc2bd6538b0c2`
- task rows: closes `12c.8.a`

## Summary

Adds a focused PG test for the updated Phase 12c tracker row
`12c.8.a`, covering two overlapping coordinator-routed DELETEs against the
same primary key.

The new test is in `src/tests/dml_concurrency.rs` so the already-large
`src/tests/dml_frontdoor.rs` does not grow further. Current file sizes:

- `src/tests/dml_concurrency.rs`: 107 lines.
- `src/tests/dml_frontdoor.rs`: 2,570 lines before this slice.

I also inspected the reviewer-flagged `12c.4` read-path schema drift row before
choosing this slice. The descriptor registration path persists coordinator and
remote shape fingerprints, but the current read target/connection/dispatch
planning path does not compare them before libpq dispatch. I did not mark
`12c.4` closed because adding that guard would be a production behavior change,
not just a coverage addition.

## Changes

- Added `src/tests/dml_concurrency.rs`.
- Included it from `src/tests/mod.rs`.
- Added `test_ec_spire_concurrent_same_pk_delete_collision_sql`.
- The test creates a SPIRE-fronted table from an external coordinator
  connection, then:
  - starts DELETE #1 in a transaction and holds the row lock briefly;
  - starts DELETE #2 against the same PK while DELETE #1 is still open;
  - asserts the two row counts sort to `[0, 1]`;
  - asserts the heap row is gone;
  - asserts the placement row is gone;
  - asserts no matching prepared xacts remain.
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to check the
  three `12c.8.a` bullets.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/mod.rs src/tests/dml_concurrency.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_concurrent_same_pk_delete_collision_sql --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- `cargo pgrx test pg18 test_ec_spire_concurrent_same_pk_delete_collision_sql`
  - Blocked before test execution by loader error:
    `undefined symbol: pg_re_throw`.

## Review Focus

- Confirm `12c.8.a` can close with this coordinator-routed DELETE collision
  fixture.
- Confirm the loser contract is represented correctly by the external SQL
  DELETE returning row count `0`.
- Confirm the cleanup assertions are sufficient for this row: no heap row, no
  placement row, and no matching prepared xact.
- Confirm the `12c.4` note is the right interpretation: current coverage cannot
  honestly close read-path schema drift until the read path has a pre-dispatch
  fingerprint guard/status surface.
