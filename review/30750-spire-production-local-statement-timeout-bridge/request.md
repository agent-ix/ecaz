# Review Request: SPIRE Production Local Statement-Timeout Bridge

## Summary

Code checkpoint: `a93088fffb92321ff2c18c9b5e0c88b09489d780`

This slice addresses packet `30749` P2 by separating local query cancellation
from local statement timeout in the production remote-cancel bridge:

- Added `local_statement_timeout` as a distinct local cancellation category,
  separate from remote-side `remote_statement_timeout`.
- When the production adapter observes PostgreSQL `InterruptPending` and
  `QueryCancelPending`, it now checks
  `get_timeout_indicator(STATEMENT_TIMEOUT, false)` to classify the local
  source without resetting PostgreSQL's timeout indicator.
- Other PostgreSQL query-cancel signals continue to map to
  `local_query_cancelled`.
- Added a PG18 backend test that dynamically schedules a local
  `STATEMENT_TIMEOUT`, proves the timeout indicator is pending, runs the normal
  production probe path, and expects `remote_transport_failed` /
  `local_statement_timeout`.
- Kept deterministic timer-triggered cancellation test-only and verified both
  transport and compact-candidate receive timer regressions still pass.
- Updated the Phase 11 task and coordinator/executor design docs to mark the
  backend interrupt bridge complete at the adapter layer. The C5 AM bridge and
  full strict/degraded fault matrix remain separate Phase 11 work.

This intentionally keeps a single production interrupt poll source:
PostgreSQL statement timeout also raises the backend query-cancel flags, so the
adapter first detects that remote work must be cancelled, then classifies the
local source by checking the statement-timeout indicator.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 prod_transport_pg`
- `cargo pgrx test pg18 local_cancel_remote_cancel`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is non-resetting `get_timeout_indicator(STATEMENT_TIMEOUT, false)` the right
  PostgreSQL boundary for local statement-timeout provenance?
- Is it acceptable to classify statement timeout inside the existing production
  `PostgresInterruptPoll` source, rather than adding a separate local-cancel
  source variant, given timeout still arrives through PostgreSQL query-cancel
  flags?
- Should the parent Phase 11.4 cancellation task now be marked complete, or stay
  open until the C5 AM bridge consumes this adapter behavior?
