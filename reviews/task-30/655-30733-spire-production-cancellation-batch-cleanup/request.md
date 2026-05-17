# Review Request: SPIRE Production Cancellation Batch Cleanup

Review the narrow pre-C5 cancellation contract slice in `0c7ab2cf`:
`Add SPIRE production cancellation batch cleanup`.

## Change

- Added an explicit production executor `Cancelled` dispatch state with
  `remote_executor_cancelled` status and `local_query_cancelled` category.
- Added cancellation counters to
  `ec_spire_remote_search_production_executor_state_summary(...)`:
  `cancelled_dispatch_count` and `first_cancellation_category`.
- Made local cancellation clear any retained `CandidateReceiveReady` compact
  candidate batch and candidate count before merge or Stage D heap resolution.
- Kept compact merge strict: cancelled dispatches cannot be interpreted as
  empty ready batches.
- Added unit coverage for all non-ready merge rejection states:
  `BlockedBeforeDispatch`, `Planned`, `TransportReady`, `TransportFailed`,
  `CandidateReceiveFailed`, and `Cancelled`.
- Cross-linked ADR-058 to the production executor stage-extension pattern and
  documented the cancel-clears-batch rule in the Stage C design/task files.

## Why

Reviewer feedback on packet 30731 called out a C2/C5 ownership risk:
a dispatch can become `CandidateReceiveReady` and retain a validated compact
batch, then receive local cancellation before merge. The production executor
must not let that stale retained batch contribute to a later compact merge or
remote heap-resolution handoff. This slice pins the rule before AM scan
integration broadens the call graph.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test production_executor_ --lib`
- `git diff --check HEAD~1..HEAD`

## Review Focus

- Confirm cancellation should clear retained compact candidate batches rather
  than freezing them.
- Confirm the new cancelled counters are the right first summary surface before
  full C2 remote cancel propagation lands.
- Confirm the all-non-ready merge rejection test locks the right strict helper
  contract for C5.
