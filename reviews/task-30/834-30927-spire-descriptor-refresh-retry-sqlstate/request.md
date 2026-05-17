---
topic: spire-descriptor-refresh-retry-sqlstate
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30927
stage: phase-12.4
status: open
---

# Review Request: SPIRE Descriptor Refresh Retry SQLSTATE

## Scope

Please review commit `ae3a4200` (`Pin SPIRE descriptor refresh retry SQLSTATE`).

This closes the Phase 12.4 tracker row for pinning a stable SQLSTATE for
descriptor refresh races and documenting the retry contract in ADR-069.

## What Changed

- `ec_spire_register_remote_node_descriptor(...)` now raises SQLSTATE `40001`
  (`serialization_failure`) when the descriptor-generation monotonic guard
  rejects a stale generation.
- The existing error message remains stable, and the error detail now tells
  callers to retry the whole coordinator write after the winning descriptor
  refresh commits.
- The stale-generation pg_test now catches the specific SQLSTATE and asserts
  the retry detail, instead of only matching panic text.
- ADR-069 and `docs/SPIRE_DIAGNOSTICS.md` document the retry contract and the
  fact that the failed transaction has not published placement state and rolls
  back its remote prepared transaction.
- The Phase 12.4 tracker marks the SQLSTATE/retry-contract row complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against
`ae3a42000f429288cbff7aca0797cd427b186ae2`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_stale_generation_rejected`

## Review Focus

- Confirm SQLSTATE `40001` is the right stable retry signal for this v1
  descriptor-refresh race.
- Confirm the retry contract is scoped correctly: retry the whole coordinator
  write, not just descriptor registration.
- Confirm the tracker row can be treated as complete with this focused
  behavior and documentation.
