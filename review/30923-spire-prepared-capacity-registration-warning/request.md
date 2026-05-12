---
topic: spire-prepared-capacity-registration-warning
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30923
stage: phase-12.4
status: open
---

# Review Request: SPIRE Prepared Capacity Registration Warning

## Scope

Please review commit `da252408` (`Warn on SPIRE remote prepared capacity at
registration`).

This completes the remaining Phase 12.4 `max_prepared_transactions` readiness
row by adding a nonblocking descriptor-registration preflight warning.

## What Changed

- `ec_spire_register_remote_node_descriptor(...)` now calls an opportunistic
  remote prepared-transaction capacity preflight.
- If the conninfo secret is missing or empty, registration warns that the
  preflight was skipped and keeps registration nonblocking.
- If the secret resolves, registration connects to the remote and runs
  `SHOW max_prepared_transactions`; connection/query/parse failures and zero
  values produce warnings.
- The registration contract adds
  `preflight_prepared_transaction_capacity` with validator
  `warn_if_remote_max_prepared_transactions_unavailable_or_zero`.
- ADR-069 and `docs/SPIRE_DIAGNOSTICS.md` document the nonblocking warning
  semantics and write-readiness implication.
- The Phase 12.4 tracker marks the descriptor-registration check/warn row
  complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against `da252408fc1a25094ef28c581c76ada15936849c`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo test --features pg18 --no-default-features prepared_transaction_registration_warning_handles_unresolved_secret`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract`

## Review Focus

- Confirm the preflight should remain nonblocking rather than rejecting
  descriptor registration.
- Confirm warning on unresolved conninfo secret is acceptable operator behavior.
- Confirm the contract row and docs accurately describe this as write-readiness
  gating, not read descriptor eligibility.
