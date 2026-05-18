# Review Request: SPIRE Production Coordinator Executor Plan

Code checkpoint: `9b1c37f7` (`Plan SPIRE production coordinator executor`)

## Summary

This planning checkpoint turns the remaining Phase 11 Stage C work into an
explicit production executor landing plan before the async/pipeline code starts.
It does not claim production readiness.

## Scope

- Adds `plan/design/spire-production-coordinator-executor.md`.
- Defines the production executor boundary separate from
  `ec_spire_remote_search_libpq_*` diagnostic SQL surfaces.
- Pins a first executor state model:
  `SpireRemoteFanoutExecutor` with per-node `SpireRemoteDispatch` state,
  bounded identity cache, limits, cancellation handles, and counters.
- Splits Stage C into reviewed build slices:
  C0 state contract, C1 async/pipeline adapter, C2 cancellation/timeouts,
  C3 production identity-cache use, C4 strict/degraded failures, C5 AM scan
  integration, and C6 operator/harness readiness.
- Updates the Phase 11 task file with concrete Stage C verification gates.
- Links ADR-058 to the new production executor plan.
- Adds the advisory-lock slot allocation log requested as optional 30720 P3
  follow-up.

## Validation

Packet-local logs live under `artifacts/` and are indexed in
`artifacts/manifest.md`.

- `git diff 37f1adbc 9b1c37f7 --check`
  - exited `0`

## Review Questions

- Does this plan draw the right boundary between diagnostic libpq SQL surfaces
  and the production AM executor?
- Is the C0-C6 landing sequence granular enough for quality review while still
  broad enough to avoid premature implementation detail?
- Are the cancellation, timeout, strict/degraded failure, and counter
  requirements strong enough before async/pipeline implementation starts?
