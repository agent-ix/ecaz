# Review Request: SPIRE Remote Candidates Test Layout

Branch: `task-30-spire`
Task row: Phase 12b.1 test subdirectory follow-up
Checkpoint scope: test-layout move only, no intended behavior change

## Summary

This checkpoint completes the remaining Phase 12b.1 test-layout row by moving
the endpoint-identity tuple-transport unit tests from
`remote_candidates/endpoint_identity.rs` into
`remote_candidates/tests/endpoint_identity.rs`.

`remote_candidates/mod.rs` includes the new test file from the same textual
scope, preserving the existing `super::*` access and test names.

## Validation

Artifacts are in `review/30993-spire-remote-candidates-test-layout/artifacts/`.

- `cargo check --no-default-features --features pg18`: pass.
- `cargo fmt --check`: pass, with existing stable-rustfmt config warnings.
- `git diff --check -- ...`: pass.
- `cargo test --no-default-features --features pg18 remote_tuple_transport`:
  pass, 3 passed / 0 failed / 1709 filtered out.

## Review Focus

- Confirm the moved test module remains behavior-equivalent.
- Confirm Phase 12b.1 now honestly has only the broad zero-behavior-change
  validation row left open.

