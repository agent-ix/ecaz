# Review Request: SPIRE Pipeline Steps Live Probe

Code checkpoint: `3d4232c9` (`Make SPIRE pipeline live probe explicit`)

## Summary

This slice addresses the Phase 8 final-review finding F-PIPELINE-2: the
consolidated `ec_spire_remote_pipeline_steps(...)` diagnostic opened live libpq
connections from its `connection_check` row. The default pipeline surface is now
dry, and the previous live probing behavior is exposed through the explicit
`ec_spire_remote_pipeline_steps_live(...)` entrypoint.

## Scope

- Refactors remote pipeline step construction into a shared row builder.
- Keeps `ec_spire_remote_pipeline_steps(...)` as the default operator entrypoint
  but makes it dry:
  - it reads descriptor and conninfo-secret presence;
  - it does not open remote libpq sockets;
  - it does not execute remote candidate, heap-candidate, or coordinator-result
    libpq probes.
- Adds `ec_spire_remote_pipeline_steps_live(...)` for the explicit socket-opening
  probe path.
- Updates `ec_spire_remote_operator_entrypoint_contract()` so the dry and live
  surfaces have distinct operator-use labels and the live entrypoint is
  reachability-tested.
- Updates `docs/SPIRE_DIAGNOSTICS.md` and Phase 10 harness notes with the dry
  default / live opt-in contract.
- Processes accepted 30715 P3 documentation feedback by pinning libpq budget GUC
  defaults and capability -> budget -> identity gate precedence in
  `plan/design/spire-libpq-executor-budget.md`.

## Validation

Packet-local logs live under `artifacts/` and are indexed in
`artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`
  - `Finished dev profile ... target(s) in 0.12s`
- `cargo fmt --check`
  - exited `0`; existing stable-rustfmt warnings for unstable options remain
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `1 passed; 0 failed; 1523 filtered out`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
  - `1 passed; 0 failed; 1523 filtered out`
- `git diff --check`
  - exited `0`

## Review Questions

- Is the split between dry default and explicit live probe the right production
  operator contract for this stage?
- Are the dry downstream step statuses (`requires_libpq_executor` with zero
  produced counts) clear enough for candidates, heap candidates, and coordinator
  result?
- Does the operator entrypoint contract wording make the cost boundary visible
  enough before this becomes runbook material?
