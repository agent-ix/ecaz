# 30773 - SPIRE Stage E Fixture Evidence Contract

## Summary

This packet reviews commit `30fc132462711e269d9fb48df08aa29dd412cdfe`
(`Document SPIRE Stage E fixture evidence contract`).

The slice addresses reviewer P3 follow-ups from packets `30770` and `30771`
before implementing the Stage E local multi-instance fixture.

Changes:

- Defines packet-local artifact names for every Stage E fault case:
  `review/{packet}/artifacts/stage_e_fault_{fault_case}_{mode}.log`.
- Defines matching lifecycle artifact names:
  `review/{packet}/artifacts/stage_e_lifecycle_{lifecycle_case}_{mode}.log`.
- States the required contents of each artifact: matrix row, injection command,
  query command, operator diagnostic row, expected status/counter delta, and
  observed status/counter delta.
- Names `ec_spire_remote_search_operator_diagnostics(...)` as the preferred
  Stage E assertion surface alongside per-case status/counter checks.
- Selects the default local simulated-network-partition mechanism:
  unreachable executor-owned conninfo plus a short
  `ec_spire.remote_search_connect_timeout_ms`, producing `connect_failed`
  without requiring `iptables` or privileged route changes.
- Updates Phase 11 task notes to record the evidence convention and diagnostic
  assertion surface.

## Key Files

- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `git diff --check -- plan/design/spire-production-coordinator-executor.md plan/tasks/task30-phase11-spire-distributed-production-parity.md`

No PostgreSQL fixture or performance run was started for this docs-only packet.

## Review Focus

- Is the artifact naming convention concrete enough for fixture reviewers to
  verify every fault/lifecycle matrix row mechanically?
- Is the required artifact content sufficient evidence for strict/degraded
  status and counter assertions?
- Is unreachable conninfo plus short connect timeout the right default local
  network-partition simulation mechanism?
- Is the operator diagnostics rollup correctly positioned as the fixture
  assertion surface without claiming fixture evidence has landed?
