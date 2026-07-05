# Review Request: SPIRE Remote Statement Timeout Taxonomy

Review the first C2 failure-taxonomy slice in `3894097a`:
`Classify SPIRE remote statement timeouts`.

## Change

- Added async production adapter classification for PostgreSQL SQLSTATE
  `57014` statement-timeout failures.
- `run_one_probe_request(...)` and compact-candidate receive now report
  `remote_statement_timeout` instead of generic `remote_query_failed` when the
  remote statement timeout cancels work.
- Reserved distinct categories for remote query cancellation and remote backend
  termination so the later C2 cancellation/fault slices do not have to overload
  `remote_query_failed`.
- Added PG18 loopback coverage for both production transport probe and
  compact-candidate receive timeout classification.
- Updated the Phase 11 task checklist with this first C2 taxonomy slice.

## Why

The production gate needs local cancel, local statement timeout, remote
statement timeout, connect timeout, and remote backend termination to remain
separate operator-visible categories. Before this slice, a remote
`statement_timeout` looked like an ordinary remote query failure in the async
production path.

This does not claim full C2 cancellation propagation. It is the taxonomy
precondition for that work.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test production_executor_ --lib`
- `cargo pgrx test pg18 test_ec_spire_prod_transport_remote_stmt_timeout`
- `cargo pgrx test pg18 test_ec_spire_prod_receive_remote_stmt_timeout`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm `remote_statement_timeout` is the right category for remote
  SQLSTATE `57014` with a statement-timeout message.
- Confirm reserving `remote_query_cancelled` and `remote_backend_terminated`
  is useful without over-claiming local-cancel propagation.
- Confirm the two PG18 loopback fixtures cover both async transport and
  compact-candidate receive paths.
