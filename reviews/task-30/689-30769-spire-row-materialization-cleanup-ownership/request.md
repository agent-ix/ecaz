# 30769 - SPIRE Row Materialization Cleanup Ownership

## Summary

This packet reviews commit `ae8b2eacd80cb829dcb3e88f3ef56cdd96292018`
(`Pin SPIRE row materialization cleanup ownership`).

The slice addresses the `30763` reviewer P3 design cleanup before the real
remote row materialization provider lands.

Changes:

- ADR-059 now cross-references ADR-064 and clarifies the boundary between
  origin-node heap visibility and coordinator AM materialization.
- ADR-064 now cross-references ADR-059 and states it begins after origin-node
  visibility has been resolved.
- ADR-064 now pins v1 cleanup ownership to epoch maintenance / operator mirror
  lifecycle outside `amrescan` and `amgettuple`.
- Phase 11 records this follow-up as packet `30769`.

No code or SQL behavior changed.

## Key Files

- `spec/adr/ADR-059-spire-remote-heap-resolution-contract.md`
- `spec/adr/ADR-064-spire-remote-row-materialization-lifecycle.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `git diff --check -- spec/adr/ADR-059-spire-remote-heap-resolution-contract.md spec/adr/ADR-064-spire-remote-row-materialization-lifecycle.md plan/tasks/task30-phase11-spire-distributed-production-parity.md`

No PostgreSQL fixture or performance run was started for this docs-only packet.

## Review Focus

- Is cleanup ownership now explicit enough for the upcoming materialized-row
  provider design?
- Does the ADR-059/ADR-064 boundary read correctly: origin visibility first,
  same-relation coordinator heap materialization second?
- Is it clear that cleanup is outside the AM scan cursor path?
