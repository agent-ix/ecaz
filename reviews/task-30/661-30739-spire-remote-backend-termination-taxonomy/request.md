# Review Request: SPIRE Remote Backend Termination Taxonomy

Review the C2 fault-taxonomy slice in `4f2e826d`:
`Classify SPIRE remote backend termination`.

## Change

- Classifies closed in-flight async remote query connections as
  `remote_backend_terminated` instead of generic `remote_query_failed`.
- Adds a PG18 loopback production transport probe fixture that terminates the
  remote backend running the probe query and asserts the distinct failure
  category.
- Updates the Phase 11 Stage C2 checklist with this classification slice.

## Why

The Phase 11 fault matrix needs remote backend termination to stay separate
from ordinary query failure, remote statement timeout, and future local cancel
propagation. This gives the transport path a concrete category before the
strict/degraded matrix starts consuming these results.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_prod_transport_backend_terminated`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm closed in-flight async query connections should map to
  `remote_backend_terminated`.
- Confirm transport-path coverage is enough for this narrow slice, with
  compact receive/backend termination matrix coverage left to the broader fault
  matrix.
