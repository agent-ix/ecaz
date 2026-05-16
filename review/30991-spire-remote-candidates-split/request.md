# Review Request: SPIRE Remote Candidates Split

Branch: `task-30-spire`
Task row: Phase 12b.1 cleanup
Checkpoint scope: structural split only, no intended behavior change

## Summary

This checkpoint starts Phase 12b by replacing the 12,971-line
`src/am/ec_spire/root/remote_candidates.rs` sink with
`src/am/ec_spire/root/remote_candidates/mod.rs` and included sibling files.

The split deliberately keeps the existing textual `include!` model inside the
`ec_spire` module scope for this first cleanup slice. That preserves existing
crate-visible symbol paths and avoids changing dispatch/operator semantics
while reducing merge friction for later 12b cleanup work.

## Layout Changes

- `src/am/ec_spire/mod.rs` now includes
  `root/remote_candidates/mod.rs`.
- New files under `src/am/ec_spire/root/remote_candidates/` split the old
  contents by concern: `sort.rs`, `vocab.rs`, `contracts.rs`, `fanout.rs`,
  `libpq_plan.rs`, `resolve.rs`, `write_payload.rs`, `dispatch.rs`,
  `payload_limits.rs`, `production_transport.rs`, `fault_matrix.rs`,
  `scan_output.rs`, `operator.rs`, `endpoint_identity.rs`, `governance.rs`,
  `payload.rs`, `executor_receive.rs`, `result_contracts.rs`, and
  `pipeline.rs`.
- The large `production_executor_state_tests` inline module moved to
  `remote_candidates/tests/production_executor_state.rs`.
- The endpoint-identity unit-test block remains co-located in
  `endpoint_identity.rs`; the tracker records that as follow-up test-layout
  cleanup rather than claiming the entire tests subdirectory row.
- Phase 12a tracker status is marked complete with the accepted final-review
  packet `30990`.
- Phase 12b tracker status is marked in progress and records the completed
  line-count and path-update subtasks.

## Validation

Artifacts are in `review/30991-spire-remote-candidates-split/artifacts/`.

- `cargo check --no-default-features --features pg18`: pass.
- `cargo fmt --check`: pass, with the existing stable-rustfmt config warnings.
- `git diff --check -- ...`: pass.
- `cargo test --no-default-features --features pg18 production_executor_state`:
  pass, 34 passed / 0 failed / 1678 filtered out.
- Per-file line-count sanity: largest new file is
  `production_transport.rs` at 1,633 lines, below the 2,500-line target.

## Review Focus

- Confirm the textual split preserves behavior and symbol visibility.
- Confirm the 12b tracker is honest about what remains open.
- Confirm the included-file names are an acceptable first slice before deeper
  module-boundary cleanup.

