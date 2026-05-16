# Review Request: SPIRE strict receive category coverage

- coder: coder1
- date: 2026-05-14
- code commit: cfab14d4 `Pin SPIRE strict receive failure categories`
- topic: SPIRE phase 12c.2 candidate receive failure coverage

## Scope

This slice adds focused unit coverage for strict candidate-receive executor summaries when phase 12a failure categories surface at receive time.

Changed file:

- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

## What Changed

Added `production_executor_strict_candidate_receive_preserves_12a_failure_categories`, which exercises both strict-mode receive failure categories currently shared with phase 12a transport/payload behavior:

- `tuple_transport_retired`
- `remote_payload_too_large`

For each category, the test verifies:

- sent/ready/failed receive dispatch counts remain correct
- `first_candidate_receive_failure_category` preserves the source category
- strict mode does not convert the failure into degraded skips
- the executor stays fail-closed with `status = remote_candidate_receive_failed`
- `next_executor_step` remains `compact_candidate_receive`

## Test File Size Discipline

The touched split file is now 1124 lines:

```text
1124 src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs
```

No large test file was expanded past the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs
cargo test --features "pg18 pg_test" --no-default-features production_executor_strict_candidate_receive_preserves_12a_failure_categories --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether this strict-mode receive coverage pins the right phase 12a category propagation contract, especially the choice to preserve `tuple_transport_retired` and `remote_payload_too_large` at candidate receive rather than normalizing them into a generic receive failure.
