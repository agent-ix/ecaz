# Review Request: SPIRE Descriptor-Race Tightening

- agent: coder1
- date: 2026-05-14
- code commit: `f0f16d1a`
- task rows: closes `12c.9.g`

## Summary

Tightened the existing coordinator INSERT descriptor-race fixture and moved
the remote-trigger/race coverage out of oversized `insert.rs` into a sibling
file.

## What Changed

Added `src/tests/insert_remote_trigger.rs` and included it from
`src/tests/mod.rs`.

Moved these existing fixtures unchanged except for the new descriptor-race
assertion:

- `test_ec_spire_insert_descriptor_race_sql`
- `test_ec_spire_trigger_multirow_commits_prepares_sql`

`test_ec_spire_insert_descriptor_race_sql` already pinned the winning
descriptor generation and winner/loser placement summary. This checkpoint adds
an assertion that no `ec_spire_remote_prepared_xact_intent` rows for the race
index remain in a non-terminal state after the loser rolls back.

Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark
`12c.9.g` complete and point at the moved fixture.

## Test File Size Discipline

The split keeps insert coverage under the target instead of adding more lines
to `insert.rs`:

```text
2317 src/tests/insert.rs
525 src/tests/insert_remote_trigger.rs
572 src/tests/custom_scan_concurrency.rs
452 src/tests/data_shape.rs
107 src/tests/dml_concurrency.rs
```

`src/tests/mod.rs` remains over target from existing repo structure; this slice
does not attempt to restructure the root include module.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/insert.rs src/tests/insert_remote_trigger.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_insert_descriptor_race_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_insert_descriptor_race_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm `12c.9.g` can close with the generation, placement, and
  non-terminal intent assertions now present in
  `test_ec_spire_insert_descriptor_race_sql`.
- Confirm the split name `insert_remote_trigger.rs` is a reasonable home for
  the descriptor-race and multi-row remote-trigger fixtures.
- Confirm leaving `src/tests/mod.rs` as-is is acceptable for this slice while
  keeping touched sibling test files under target.
