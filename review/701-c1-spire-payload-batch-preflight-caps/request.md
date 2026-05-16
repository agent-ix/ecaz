---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-payload-batch-preflight-caps
code_commit: cc7bd86f
---

# Review Request: Remote Payload Batch Preflight Caps

## Summary

Added a focused pure-Rust coverage pin for the 12c.2.a per-batch payload cap:

- `production_receive_adapters_reject_selected_pid_batches_before_connection`
  sends 65 selected PIDs through both production receive adapters, exceeding
  the test default `ec_spire.max_remote_payload_rows_per_batch` of 64.
- The requests carry an intentionally invalid conninfo string, and the test
  asserts both adapters return `remote_payload_too_large` without requiring a
  live connection.
- Candidate receive is asserted as
  `remote_candidate_receive_failed` with no candidate batch.
- Remote heap receive is asserted as `remote_heap_resolution_failed` with no
  returned candidates.

This pins that the per-batch cap fires before libpq connection setup and before
remote result allocation for candidate and heap receive paths.

## Scope

Changed:

- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

This is partial coverage for 12c.2.a:

- Covered: selected-PID per-batch cap preflight for strict production candidate
  receive and remote heap receive adapters.
- Already present nearby: row-byte cap helper rejection and degraded skip report
  hint for `remote_payload_too_large`.
- Not covered here: a live CustomScan strict/degraded fixture with a remote
  endpoint returning an oversized typed row.

File-size check: `production_executor_state.rs` is 1180 lines after this slice.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`
- `cargo test --no-default-features --features pg18 production_receive_adapters_reject_selected_pid_batches_before_connection --no-run`
  - Existing unused-import warnings in `src/am/mod.rs`.

Attempted runtime execution:

- `cargo test --no-default-features --features pg18 production_receive_adapters_reject_selected_pid_batches_before_connection`
  - The test binary linked, then exited locally with the existing pgrx runtime
    loader issue: `undefined symbol: BufferBlocks`.

## Review Focus

Please check whether this is the right level to pin the per-batch cap ordering
before adding the harder live remote oversized-payload fixture.
