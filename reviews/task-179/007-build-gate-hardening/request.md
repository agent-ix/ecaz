---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 007 build-gate hardening

This packet hardens the durable coordinator build gate
(`src/am/ec_distann/build_gate.rs`, `build_coordinator.rs`) and directly
answers the build-gate findings in
`006-publication-and-retention/feedback/2026-07-11-05-reviewer.md` (the outside
review of the post-request lane commits, which reviewed the original gate
foundation `28b99f151`).

## Commits

- `965fa1dfd` — preload / inheritance / global-utility hardening.
- `0ee8d49b3` — ExecutorStart enforcement (closes the P1 cached-plan bypass).

Rebased onto outside-review commit `7296ca106`. Artifacts + per-artifact
provenance in `artifacts/manifest.md`.

## Disposition of packet-006 seq-05 build_gate findings

- **P1 — cached-plan bypass (plan-time-only enforcement): FIXED (`0ee8d49b3`).**
  Source-DML rejection moved from the planner hook to an `ExecutorStart` hook
  over `plannedstmt->resultRelations`. A prepared/PL-pgSQL/FK-trigger plan built
  before the registration committed now re-enters the gate on execution through
  `CheckCachedPlan`/`AcquireExecutorLocks`. `resultRelations` is the global
  ModifyTable target list (covers data-modifying CTEs in a read-only SELECT), so
  no `commandType` filter is needed and pure reads never invoke the mask. Live
  regression: a parameterless INSERT plan cached before the gate is rejected
  with `EC_BUILD_STATE` post-gate (not `EC_GENERATION_MISSING`, which would mean
  it reached aminsert — i.e. bypassed the gate), plus a positive control on an
  unrelated table. The logical-replication `ExecSimpleRelationInsert` residual
  is documented in code as out of scope.
- **P2-1 — DROP EXTENSION / uninstalled-DB DML breakage: FIXED (`965fa1dfd`).**
  `invoke_gate_mask` returns 0 when the `ecaz` extension is not installed in the
  current database (checked before any `extension_relation_name` resolution), so
  a shared-preloaded `.so` no longer breaks DML in databases without the
  extension or after `DROP EXTENSION`. Live test:
  `test_distann_preloaded_hook_passes_through_without_extension`.
- **P2-2 — lock-upgrade deadlock: FIXED (`965fa1dfd`).** The utility hook now
  resolves each named relation with the operation's true conflicting lockmode
  (AccessExclusive / RowExclusive) instead of AccessShareLock-then-escalate, so
  concurrent DROP/ALTER/TRUNCATE of the same table queue as in stock
  PostgreSQL.
- **P2-3 — fail-open dependency-traversal bypasses: MOSTLY FIXED (`965fa1dfd`).**
  Added gating for `DROP SCHEMA ... CASCADE`, `DROP OWNED BY`, `REINDEX
  SCHEMA/DATABASE`, `ALTER ... ALL IN TABLESPACE`, `RENAME`, `SET SCHEMA`, and
  the bare CLUSTER/VACUUM-FULL whole-registry check, all serialized under a
  session-exclusive advisory lock across `standard_ProcessUtility`. **Residual:**
  `TRUNCATE other CASCADE` truncating a gated source that does not appear in
  `stmt.relations` is not yet covered — flagged as a follow-up.
- **P2-4 — per-DML hot-path cost: PARTIALLY ADDRESSED (`965fa1dfd`).** The gate
  helper OID is backend-cached with PROCOID syscache invalidation, and reads
  (empty `resultRelations`) never call the mask. **Residual:** a shared "any
  live registration?" fast path and an A/B latency data point are still owed
  before closeout.
- **P3-1 (no gate-clearing surface at this commit):** still true — release is
  modeled with raw `UPDATE ... state='Published'`; the real lifecycle release
  path lands with the coordinator decision/recovery slice. Stated here
  explicitly.
- **P3-4 (missing positive control + prepared-statement scenario):** both now
  present in the competing-backend test.

## What this packet does NOT close

Coordinator build-to-Ready, decision/recovery/retirement, topology endpoints,
physical read path, the real three-instance fixture, and the P2-4 latency A/B
remain open in their owning Task 179 packets. The remaining seq-05 P3s
(WARNING-on-skip for OID-reuse disarm, REPEATABLE-READ snapshot note) are
tracked for a follow-up gate touch.

## Validation

- `cargo check` + strict `cargo clippy` (`pg18 pg_test`, `-D warnings`) — pass
  at `0ee8d49b3`.
- `cargo pgrx test pg18 test_distann_begin_build` — 3/3 pass (competing-backend
  incl. cached-plan P1 regression, inherited-source rejection, lock-lifecycle).
- `cargo pgrx test pg18 test_distann_preloaded_hook_passes_through_without_extension`
  — 1/1 pass.

Please review `965fa1dfd` and `0ee8d49b3`. Leaving the request open for outside
review.
