# Review Request: SPIRE Remote Failure Taxonomy Doc Clarity

Review the P3 cleanup in `92e4d64d`:
`Clarify SPIRE remote failure taxonomy blockers`.

## Change

- Added comments near `production_remote_query_failure_category(...)` spelling
  out the closed-connection and SQLSTATE `57014` message-text classification
  rules.
- Added C3 production identity-cache reuse to the Stage C milestone blocker
  list in `plan/design/spire-production-coordinator-executor.md`.

## Why

This addresses reviewer P3 follow-ons from packets 30737-30739 without changing
runtime behavior. The taxonomy comments make explicit which branches map to
`remote_backend_terminated`, `remote_statement_timeout`, and
`remote_query_cancelled`; the design note now names C3 alongside C2/C4/C5.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm the classification comments are accurate and not too broad.
- Confirm naming C3 explicitly in the production gate blocker list is the right
  wording.
