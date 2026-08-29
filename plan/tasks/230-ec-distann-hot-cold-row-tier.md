# Task 230: ec_distann Hot/Cold Vertical Row Tier

> **Tracking moved to GitHub (2026-08-29):** [agent-ix/ecaz#96](https://github.com/agent-ix/ecaz/issues/96)
> on [Project 19](https://github.com/orgs/agent-ix/projects/19), under EPIC #95.
> The Status header below is frozen; status updates land on the issue.
> Review packets remain under `reviews/task-230/`.

Status: **proposed — operator-selected mandatory prototype, after Task 229**
(2026-08-22). Priority: P1 storage/retrieval latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
ARCH-16. This task evaluates a vertical layout independently of Task 229's
covering sidecar; it is not skipped if Task 229 wins.

## Why

The frozen generation row tier mixes attributes with very different read
behavior: the exact vector is read by ANN expansion/rerank, small identifiers
and scalars dominate common result projections, and large arrays or toasted
payloads are normally cold. Keeping all of them in one PostgreSQL heap tuple
couples hot retrieval to tuple deformation and TOAST state for unrelated
columns.

A vertical split can keep the exact vector and a deliberately small scalar
cover in a dense hot tier while retaining arbitrary PostgreSQL values in a cold
tier. Unlike Task 229, this changes the authoritative row-tier representation
and can benefit both graph expansion and result materialization.

## Goal

Implement and benchmark an opt-in hot/cold generation layout that reads the
exact vector and selected scalar attributes without touching cold payload
storage, while reconstructing arbitrary SQL rows exactly when cold attributes
are required.

## Entry conditions

1. Tasks 222--224 are review-closed and define the frozen unstacked control.
2. Task 229 has completed its full matrix, but its sidecar is not enabled in
   this candidate arm.
3. The hot/cold attribute contract and expected storage amplification are
   reviewed before the persisted format is written.

## Required implementation

### P1 — Vertical format

- Define a generation-owned hot tier containing vec_id, tombstone/visibility
  metadata, the full-precision exact vector, and an explicitly bounded scalar
  cover.
- Define a cold tier containing every remaining source attribute with enough
  attnum/type metadata to reconstruct the original row descriptor exactly.
- Store each logical attribute in exactly one authoritative tier. Do not keep a
  second exact vector or duplicate the full source row merely to simplify
  fallback.
- Give graph records a versioned locator that resolves the hot row and its cold
  counterpart without interpreting a cross-node CTID.

### P2 — Read and mutation paths

- Expansion/rerank reads only the hot exact-vector surface.
- Result materialization reads only the tiers required by Task 222's mask and
  merges values in original attnum order.
- Task 167 insert/replacement/delete operations update hot, cold, graph, and
  directory state atomically under the existing intent/retry rules.
- Bind both tier digests and bytes into receipts, manifest/fingerprint,
  topology, storage accounting, retirement, and reclaim.
- Preserve rebuild rollback and a readable fallback for older row-heap
  generations.

### P3 — Evidence

- Cover id-only, hot-scalar, exact-vector projection, cold-scalar, mixed
  hot/cold, `SELECT *`, NULL, external TOAST, qual-only, deepening, rescan,
  mutation, restart, retirement, and owner failure.
- Run isolated row-heap versus hot/cold A/B at 10k, 50k, and 100k using
  `ecaz bench suite`. Do not combine Task 229's sidecar or Task 231's fixed
  graph blocks with this candidate.
- Compare arms at matched fixture position, or use a preregistered
  counterbalanced envelope that separates position/warmth from the candidate.
  Never compare a fresh-build control only against a reused candidate.
- Report exact-vector reads, hot/cold tuple and block reads, detoast/send work,
  payload bytes, graph expansion stages, end-to-end latency/tails, recall,
  build/DML cost, per-tier bytes, and conformance.

## Decision rule

An implemented prototype and full 10k/50k/100k matrix are mandatory. Close
PROMOTE or STOP based on measured end-to-end retrieval, storage, construction,
and mutation effects. Continue to Tasks 231 and 232 regardless of outcome.

## Non-goals

- A fully columnar representation; Task 232 owns that design.
- Fixed-stride graph-node extents; Task 231 owns them.
- Replicating hot rows at the coordinator.
- Stacking the Task 229 sidecar into the candidate arm.

## Acceptance

1. Every source attribute has one authoritative tier and exact row
   reconstruction is test-pinned.
2. Exact-vector reads avoid the cold tier and are observable in counters.
3. DML and generation lifecycle behavior remains transactional and fail-closed.
4. Full-scale packet evidence supports a reviewed PROMOTE or STOP disposition.

## Required review packets

1. `reviews/task-230/001-plan/`
2. `reviews/task-230/002-format-and-read-path/`
3. `reviews/task-230/003-lifecycle-and-dml/`
4. `reviews/task-230/004-full-scale-decision/`

## References

- Tasks 222--224 and 229
- FR-076, FR-078, FR-079, FR-082, FR-083
- NFR-016, NFR-018, NFR-021, NFR-022
- PostgreSQL TOAST storage behavior

