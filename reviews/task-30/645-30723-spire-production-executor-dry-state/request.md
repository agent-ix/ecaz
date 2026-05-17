# Review Request: SPIRE Production Executor Dry State

Code checkpoint: `ab64003b` (`Add SPIRE production executor dry state`)

## Summary

This is the first Phase 11 Stage C implementation slice after the production
coordinator executor plan. It adds a dry production fanout state summary that
can be built from admitted/blocked remote dispatch planning data without
resolving conninfo secrets, opening libpq sockets, or querying endpoint
identity.

## Scope

- Adds `SpireRemoteFanoutExecutor` and per-dispatch production state scaffolding.
- Adds `ec_spire_remote_search_production_executor_state_summary(...)`.
- Reports C0 counters for planned dispatches, pre-dispatch blockers, planned
  and blocked PIDs, secret lookups, socket opens, and endpoint identity probes.
- Keeps admitted dispatches at
  `requires_production_transport_adapter` /
  `production_transport_adapter` until the C1 async/pipeline transport lands.
- Preserves pre-dispatch blockers such as `remote_executor_overload` before
  secret lookup.
- Adds the new dry-state entrypoint to the operator contract and
  `docs/SPIRE_DIAGNOSTICS.md`.
- Updates the Phase 11 task file to mark the C0 state scaffolding and dry
  summary as landed.

## Validation

Packet-local logs live under `artifacts/` and are indexed in
`artifacts/manifest.md`.

- `git diff 38c807ec ab64003b --check`
  - exited `0`
- `cargo fmt --check`
  - exited `0`; existing stable-rustfmt warnings for unstable options remain
- `cargo check --no-default-features --features pg18`
  - `Finished dev profile ... target(s) in 0.12s`
- `cargo test production_executor_state --lib`
  - Rust unit tests:
    `production_executor_state_keeps_admitted_dispatches_dry ... ok`
  - Rust unit tests:
    `production_executor_state_preserves_pre_dispatch_blocker ... ok`
  - PG18 test:
    `pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
  - `3 passed; 0 failed`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
  - `1 passed; 0 failed`

## Review Questions

- Is the dry production-state summary a clean C0 bridge from diagnostic
  dispatch planning to the future production executor?
- Are the C0 counters sufficient to prove no secret lookup, socket open, or
  endpoint identity probe occurs before C1 transport work?
- Are the new status and next-step labels clear enough for operators and future
  Stage C diagnostics?
