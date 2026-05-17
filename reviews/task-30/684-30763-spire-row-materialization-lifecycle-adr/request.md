# 30763 - SPIRE Row Materialization Lifecycle ADR

## Summary

This packet reviews commit `e4843e2aa9f288e8fae8392f0eb4d674650aa7db`
(`Propose SPIRE remote row materialization lifecycle`).

The slice responds to the `30761` reviewer P2 before implementation of remote
row materialization. It adds ADR-064, which pins the v1 lifecycle for
remote-origin rows returned through the PostgreSQL index AM:

- no scan-time heap writes from `amrescan` / `amgettuple`;
- no per-query/per-cursor temp, scratch, tuplestore, or in-memory proxy rows;
- AM-deliverable remote-origin rows require a pre-existing coordinator-visible
  heap row in the same scanned relation;
- materialized row lifetime is epoch/MVCC scoped, with cleanup outside the scan
  cursor path.

This intentionally narrows the implementation path: remote-origin AM delivery
depends on a coordinator heap mirror/replication lifecycle. Deployments that do
not want same-relation materialized rows need a future FDW/custom-scan tuple
delivery surface instead of the v1 index AM path.

## Key Files

- `spec/adr/ADR-064-spire-remote-row-materialization-lifecycle.md`
- `spec/adr/index.md`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `git diff --check -- <changed docs>`

No code or SQL behavior changed in this packet.

## Review Focus

- Is rejecting per-cursor/temp/scratch proxy rows the right interpretation of
  PostgreSQL's `xs_heaptid` relation identity contract?
- Is epoch-scoped same-relation coordinator heap materialization the right v1
  lifecycle before implementation starts?
- Are the implementation consequences explicit enough: no scan-time heap writes,
  validate visible same-relation materialized TIDs, and keep FDW/custom-scan
  separate?
