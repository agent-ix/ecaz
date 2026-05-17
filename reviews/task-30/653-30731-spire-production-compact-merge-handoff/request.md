# Review Request: SPIRE Production Compact Merge Handoff

- code commit: `1c0796753dc1224415d5ce0cfd53e2b68799101d`
- reviewer focus: C5 precursor, executor-owned compact candidate handoff
- phase: Phase 11 Stage C/D boundary

## Summary

This checkpoint makes the production executor retain ready compact-candidate
batches, then exposes a strict merge handoff that consumes only
`CandidateReceiveReady` dispatches.

What changed:

- `SpireRemoteProductionDispatch` now stores the ready
  `SpireRemoteSearchCandidateBatch`.
- ready receive transitions preserve the batch for Stage D handoff.
- failed receive transitions clear the batch.
- `SpireRemoteFanoutExecutor::merge_ready_candidate_batches(...)` validates and
  merges only ready batches using the existing remote candidate merge contract.
- merge is rejected if any dispatch is still planned, transport-ready,
  transport-failed, candidate-receive-failed, or blocked before dispatch.
- the hard-coded `remote_heap_resolution` string is now a named executor-step
  constant.

This does not yet wire the AM scan callback to run production fanout. It removes
the bookkeeping gap that would otherwise force C5 to duplicate compact receive
state outside the executor.

## Validation

Artifacts are packet-local under `artifacts/`.

- `cargo fmt --check`
  - log: `artifacts/cargo-fmt-check.log`
  - result: pass, with existing rustfmt stable-channel warnings.
- `cargo check --no-default-features --features pg18`
  - log: `artifacts/cargo-check-pg18.log`
  - result: pass.
- `cargo test --no-default-features --features pg18 production_executor_`
  - log: `artifacts/cargo-test-production-executor.log`
  - result: `11 passed; 0 failed`.
- `git diff 95b0fda728b6d5f8462424414f1bfa6d54720fb0 1c0796753dc1224415d5ce0cfd53e2b68799101d --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass.

## Requested Review

Please check the C5 handoff shape:

- Is storing `SpireRemoteSearchCandidateBatch` on the ready dispatch the right
  owner boundary for AM scan integration?
- Should strict merge reject all non-ready dispatch states as implemented, or
  should any pre-C4 degraded shape be tolerated?
- Is the compact merge helper narrow enough, or should the next slice expose a
  SQL-visible summary before AM scan wiring?
