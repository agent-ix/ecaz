# Review Request: SPIRE Remote Libpq Executor Boundary

Code checkpoint: `e463120c` (`Document SPIRE remote libpq executor boundary`)

## Scope

- Advances Phase 10.5 by deciding the role of the current SQL-visible libpq
  executor.
- Adds ADR-058, which keeps the current executor diagnostic/operator-only
  rather than treating it as the production AM remote query path.
- Records the current limitations: blocking `postgres::Client` calls,
  per-query connections, serial dispatch, no pipeline mode, no AM-owned
  cancellation/timeout/final-row-delivery model.
- Marks the Phase 10.5 raw-conninfo and receive-batch-validation items
  complete for the diagnostic executor, while deferring production remote
  execution to a future ADR/checkpoint.
- Updates the remote-node design note and Phase 10 task file to point at the
  accepted boundary.

## Validation

- `git diff --check`
- Tests not run; this is a documentation-only checkpoint.

## Review Focus

- Confirm the diagnostic-only boundary is clear enough to prevent performance
  overclaiming from the SQL-visible libpq functions.
- Confirm the future production-executor requirements cover the missing pieces:
  concurrent dispatch, pipeline/async receive, bounded fanout, timeouts,
  cancellation, identity validation, fail-closed behavior, and final row
  delivery.
- Confirm marking raw conninfo and receive validation complete is scoped to the
  diagnostic executor and not presented as production readiness.
