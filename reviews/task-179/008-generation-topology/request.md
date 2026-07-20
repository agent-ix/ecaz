---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 008 generation topology inspection

Implements the build-id-selected physical topology endpoint
`ec_distann_generation_topology(regclass, uuid)` (FR-078:764-804), which the
coordinator uses at T3 to verify a Ready generation before persisting a publish
decision, and operators use to audit an in-progress build.

## Commit

- `b64a35e4e` — `ec_distann_generation_topology` + `diagnose_physical_generation`.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping (FR-078:764-804, FR-078-CON-4, FR-078-AC-10)

- **15-column output**: `node_id, state, record_count, row_count,
  owned_vec_id_digest, graph_digest, row_tier_digest, non_owned_live_count,
  non_owned_tombstone_count, orphan_record_count, orphan_row_count, graph_bytes,
  row_tier_bytes, directory_bytes, control_index_bytes`.
- **Digests recomputed from physical relations, never manifest fields**:
  `owned_vec_id_digest` = `SHA-256("ec_distann_owned_vec_ids_v1\0" ||
  u64_le(vec_id)...)` over owned vec_ids in ascending order; `graph_digest` /
  `row_tier_digest` use the seal domains and **equal the Ready receipt** when the
  generation is clean (asserted live). Sizes use the exact Ready-receipt size
  functions; `control_index_bytes` reports the logical control relation
  separately (FR-078:794-796).
- **Diagnostic, not strict**: unlike `scan_physical_generation` (which errors on
  any non-owned/tombstone/mismatch), `diagnose_physical_generation` classifies
  and counts — non-owned live/tombstoned, orphaned records (owned-live with no
  co-located row), and orphaned rows — so it can report diagnostics rather than
  hard-coded zeros. A clean Ready generation reports all-zero residue.
- **State acceptance**: Building/Ready reported by build id; Published/retained
  Retired/Reclaimed/Aborted/absent yield no rows (they are inspected by
  fingerprint through `ec_distann_epoch_topology`).
- **Locking**: AccessShareLock on the graph and row-tier heaps and an OID lock
  on the directory (a unique index, not a heap) so a concurrent retirement
  reclaim — which drops storage under AccessExclusiveLock — cannot delete
  relations mid-inspection.

## Deliberately deferred

`ec_distann_epoch_topology(regclass, bytea)` (by-fingerprint, Published/retained
Retired) is **not** in this packet: resolving a fingerprint to its Published/
retained generation requires the active-pointer / publication-decision schema
that the coordinator decision slice has not yet built. It lands with that slice,
reusing `diagnose_physical_generation` unchanged. The fingerprint-version
rejection (`EC_EPOCH_FINGERPRINT_VERSION`) and Reclaimed fail-closed
(`EC_GENERATION_MISSING`) will be added there.

## Validation

- `cargo check` + strict `cargo clippy` (`pg18 pg_test`, `-D warnings`) — pass at
  `b64a35e4e`.
- `cargo pgrx test pg18 test_distann_generation_topology_reports_ready_and_building`
  — 1/1 pass (Building empty, Building staged, Ready with digests equal to the
  sealed Ready receipt, unknown build id → no rows).

Leaving the request open for outside review.
