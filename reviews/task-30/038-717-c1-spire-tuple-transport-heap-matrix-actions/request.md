# Review Request: SPIRE 12c Tuple Transport Heap Matrix Actions

- agent: coder1
- date: 2026-05-14
- code commit: `c33ce1b81268ac1ee0e27a9bf447d029d1707662`
- task rows: partial support for `12c.13.a`; partial support for `12c.2.b`

## Summary

Adds executor-state coverage for the `tuple_transport_retired` failure category
at the remote-heap stage.

This complements the existing endpoint-identity unit test that rejects legacy
`json_tuple_payload_v1` production SQL and the existing degraded skip hint
coverage. It does not claim to close `12c.2.b`, because that row still asks for
a live remote advertisement fixture.

The test stays in
`src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`,
now 1,353 lines.

## Changes

- Added heap receive result helpers for production executor-state tests.
- Added `production_executor_heap_tuple_transport_retired_matrix_actions`.
- Strict mode applies a failed heap receive for node 2 and a ready heap receive
  for node 3, then asserts:
  - one heap-ready and one heap-failed dispatch
  - node 2 failure category is `tuple_transport_retired`
  - no degraded skip is recorded
  - summary fails closed with `remote_heap_resolution_failed`
- Degraded mode applies the same heap receive outcomes and asserts:
  - node 2 is skipped/reported
  - node 3 remains heap-ready
  - summary status is `degraded_ready`
  - degraded skip report carries the tuple-transport upgrade hint

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`
  - Passed.
- `cargo test --no-default-features --features pg18 production_executor_heap_tuple_transport_retired_matrix_actions --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.

## Review Focus

- Confirm this is useful incremental `12c.13.a` executor-action coverage.
- Confirm it should remain only partial for `12c.2.b` until a live remote
  advertisement fixture exists.
- Confirm the strict fail-closed and degraded skip/report expectations match
  the production executor state machine.
