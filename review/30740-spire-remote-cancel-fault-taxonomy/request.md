# Review Request: SPIRE Remote Cancel Fault Taxonomy

Review the C2 taxonomy coverage slice in `a1c02ce9`:
`Cover SPIRE remote cancel fault taxonomy`.

## Change

- Added PG18 coverage proving `remote_query_cancelled` is emitted on the async
  production transport path.
- Added PG18 coverage proving `remote_query_cancelled` is emitted on the
  compact-candidate receive path.
- Added the receive-path mirror fixture for `remote_backend_terminated`, closing
  the optional follow-up from packet 30739.
- Updated the Phase 11 Stage C2 checklist with the new fault-taxonomy coverage.

## Why

Packets 30738 and 30739 reserved/claimed the key C2 fault categories. This
slice makes `remote_query_cancelled` a claimed category and gives
`remote_backend_terminated` symmetric receive-path coverage before the broader
strict/degraded fault matrix consumes these categories.

This still does not claim local cancellation propagation to in-flight remote
work. It proves the remote-side categories the later propagation slice will
need.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_query_cancelled`
- `cargo pgrx test pg18 test_ec_spire_prod_receive_backend_terminated`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm the self-cancel loopback fixtures are a reasonable way to claim
  `remote_query_cancelled` separately from `remote_statement_timeout`.
- Confirm the receive-path backend-termination fixture completes the symmetric
  coverage requested in packet 30739 P3.
- Confirm the slice remains taxonomy-only and does not over-claim local
  cancellation propagation.
