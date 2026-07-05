# Review Request: SPIRE Production Candidate Receive State

- code commit: `8310e767d3b7f0241daca0aa79d06d66793ef2f2`
- reviewer focus: production executor state after compact candidate receive
- phase: Phase 11 Stage C, C1 production fanout executor

## Summary

This checkpoint wires compact-candidate receive outcomes into the production
executor state machine after the transport adapter stage.

The executor now tracks candidate receive as explicit dispatch states:

- `TransportReady` dispatches are counted as candidate-receive pending.
- ready candidate receive results transition to `CandidateReceiveReady`,
  record candidate-row counts, and advance the executor to
  `remote_heap_resolution` with status `requires_remote_heap_resolution`.
- failed candidate receive results transition to `CandidateReceiveFailed`,
  expose `remote_candidate_receive_failed`, and preserve the first failure
  category for diagnostics.
- receive results that do not match a transport-ready dispatch are rejected.

The SQL dry summary now exposes:

- `candidate_receive_pending_dispatch_count`
- `candidate_receive_sent_dispatch_count`
- `candidate_receive_ready_dispatch_count`
- `candidate_receive_failed_dispatch_count`
- `candidate_row_count`
- `first_candidate_receive_failure_category`

## What Changed

- Extended `SpireRemoteProductionExecutorStateSummaryRow` with candidate
  receive counters and the first candidate receive failure category.
- Added candidate receive state transitions to `SpireRemoteFanoutExecutor`.
- Added test-only summary helper that composes dispatch rows, transport rows,
  and candidate receive results.
- Added focused unit coverage for ready receive, failed receive, and receive
  before transport.
- Updated the PG18 dry SQL summary test to assert the new columns.
- Marked the Phase 11 task subitem complete.

## Validation

Artifacts are packet-local under `artifacts/`.

- `cargo fmt --check`
  - log: `artifacts/cargo-fmt-check.log`
  - result: pass, with existing rustfmt stable-channel warnings.
- `cargo check --no-default-features --features pg18`
  - log: `artifacts/cargo-check-pg18.log`
  - result: pass.
- `cargo test --no-default-features --features pg18 production_executor_state_`
  - log: `artifacts/cargo-test-production-executor-state.log`
  - result: `9 passed; 0 failed`.
- `git diff a73b5d9c520552ee7009bd9c4de9ba5fedea791f 8310e767d3b7f0241daca0aa79d06d66793ef2f2 --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass.

## Requested Review

Please check whether the production executor state model is still composable as
we add remote heap resolution and final merge:

- Are the pending, sent, ready, failed counters the right shape for another
  executor stage?
- Should `remote_heap_resolution` be promoted to a named constant before the
  next slice?
- Are the failure precedence rules correct when one node fails receive while
  another already has candidates?
- Is it acceptable that state tests treat receive batches as already validated
  adapter output, while adapter validation is covered in packet 30727?
