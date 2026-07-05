# Review Request: SPIRE Production Receive State Adapter

Review the Stage C receive-state wiring slice in `dd8c7ab0`:
`Wire SPIRE production receive into executor state`.

## Change

- Added `SpireRemoteFanoutExecutor::run_compact_candidate_receive(...)`.
- The method builds internal receive requests from `TransportReady` dispatches,
  runs the async production candidate receive adapter, and applies results back
  into the same executor state machine.
- Missing-secret failures from request building stay per-dispatch
  `CandidateReceiveFailed`; adapter failures also apply as per-dispatch
  candidate receive results.
- Updated the Phase 11 checklist to split "compact receive production state is
  wired" from the still-open AM scan integration item.

## Why

Packet 30734 made the executor state capable of building receive requests. This
slice closes the next gap: production state can now execute compact candidate
receive and own the resulting `CandidateReceiveReady` or
`CandidateReceiveFailed` transitions. AM scan integration can call this state
method instead of duplicating request/result bookkeeping.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test production_executor_ --lib`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm `run_compact_candidate_receive(...)` is the right executor-owned
  boundary before AM scan integration.
- Confirm adapter failures should apply through the same
  `CandidateReceiveFailed` transition as decoded receive failures.
- Confirm the remaining checklist split is honest: compact receive production
  state is wired, AM scan integration and final rows remain open.
