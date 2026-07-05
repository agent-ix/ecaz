# Review Request: SPIRE Remote Endpoint Strict Rejection

## Scope

This packet covers commit `9107021600e15fd657c0275838007516d481deb9`.

The slice adds PG18 loopback coverage for the Phase 11 Stage B fail-closed
gate: a libpq remote executor must reject a remote endpoint whose identity is
not `ready` before any candidate can enter the merge path. The new negative
fixture builds a loopback remote SPIRE index with the default non-RaBitQ storage
format and expects the executor to fail with:

`ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready`

The existing ready loopback fixture remains the positive guardrail and uses
`storage_format = 'rabitq'`.

## Files

- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_libpq_executor_rejects_non_ready_endpoint`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `git diff --check`

Raw logs are in `artifacts/`; see `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the negative loopback fixture proves the strict endpoint ready gate
  at the production libpq receive boundary, not only the diagnostic SQL
  endpoint.
- Confirm the task-plan wording does not overclaim broader Stage B completion.
- Confirm this is the right prerequisite before adding degraded-mode per-node
  skip reporting for endpoint/fingerprint/opclass mismatches.
