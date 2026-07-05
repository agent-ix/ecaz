# Review Request: SPIRE Production Receive Request State

Review the Stage C state-boundary slice in `cd7b1919`:
`Add SPIRE production receive request state`.

## Change

- Extended `SpireRemoteProductionDispatch` to retain the selected PIDs,
  `conninfo_secret_name`, and remote index regclass copied from the dispatch
  plan row.
- Added `SpireRemoteFanoutExecutor::compact_candidate_receive_requests(...)`,
  which builds internal async receive-adapter requests from `TransportReady`
  dispatches without re-reading diagnostic rows or keeping parallel AM scan
  bookkeeping.
- Kept raw conninfo inside the executor boundary: dispatch state stores only
  `conninfo_secret_name`; raw conninfo is resolved only when building internal
  receive adapter requests.
- Isolated missing secret resolution as a per-dispatch
  `CandidateReceiveFailed` transition with failure category
  `requires_conninfo_secret_resolution`.
- Changed production receive request `consistency_mode` from `&'static str` to
  owned `String`, so future scan-path callers are not forced through static
  literals.
- Marked the Phase 11 Stage C checklist item for dispatch-owned receive
  request state.

## Why

C5 should consume the production executor state, not parallel vectors assembled
beside it and not diagnostic SQL-visible receive rows. This slice makes the
executor state composable enough for the next step: run compact candidate
receive from `TransportReady` dispatches and apply the returned batches back
into the same state machine.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test production_executor_ --lib`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm the executor should own selected PIDs, secret reference, and remote
  regclass as dispatch state.
- Confirm resolving raw conninfo only during internal receive-request build
  preserves the no-raw-conninfo diagnostic boundary.
- Confirm missing secret isolation should become `CandidateReceiveFailed`
  rather than aborting the whole receive request build.
