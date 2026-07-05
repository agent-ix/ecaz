---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-prepared-xact-intent-transitions
code_commit: f019f505
---

# Review Request: Prepared Xact Intent Transitions

## Summary

Added a focused 12c.5.b state-machine coverage pin for SPIRE remote prepared
transaction intents:

- Introduced explicit transition contexts:
  - `RemotePrepareAck`
  - `LocalCommitRecorded`
  - `ReaperRollback`
- Added `coordinator_prepared_xact_intent_transition_is_valid(...)`.
- Routed existing prepare-ack, commit-local, and reaper rollback mark calls
  through the explicit context.
- Added a `#[cfg(test)]` invariant in `coordinator_prepared_xact_intent_mark`
  so test builds reject invalid context/state transitions if the current intent
  row exists.
- Added `prepared_transaction_intent_transitions_cannot_bypass_prepare_ack`.

The new test asserts:

- `prepare_requested -> prepare_acked` is valid only for remote prepare ACK.
- `prepare_acked -> commit_local` is valid only for local commit recording.
- `prepare_requested/prepare_acked -> rollback_local` is valid for the reaper.
- `prepare_requested -> commit_local` is invalid.
- silent/local `prepare_requested -> rollback_local` is invalid.
- `commit_local -> rollback_local` is invalid even for the reaper.

## Scope

Changed:

- `src/am/ec_spire/coordinator/remote_candidates/resolve.rs`
- `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`
- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

This covers 12c.5.b intent state-machine invariants. It does not cover the
12c.5.a live in-doubt reaper fixture (`prepare_acked -> commit_local` crash
window), which still needs a cross-session prepared-transaction test.

File-size check:

- `resolve.rs`: 380 lines.
- `write_payload.rs`: 845 lines.
- `production_executor_state.rs`: 1225 lines.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/am/ec_spire/coordinator/remote_candidates/resolve.rs src/am/ec_spire/coordinator/remote_candidates/write_payload.rs src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`
- `cargo test --no-default-features --features pg18 prepared_transaction_intent_transitions_cannot_bypass_prepare_ack --no-run`
  - Existing unused-import warning in `src/am/mod.rs`.

## Review Focus

Please check whether the transition contexts model the intended write/reaper
state machine clearly enough before the live 12c.5.a reaper fixture lands.
