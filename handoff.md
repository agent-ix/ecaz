# Task 30 Phase 4 Handoff

You are continuing Task 30 SPIRE on branch
`task30-spire-partition-object-spec`.

## Current State

- Starting point before this handoff: `59e37fa2c2628a9efa33b8f85ecd4ab585f234c5`.
- Worktree was clean before writing this handoff.
- Phase 3 recursion is complete for the current milestone.
- Reviewer closeout feedback was handled in:
  - `9a00540d` - reconciled Phase 3 plan follow-ups.
  - `da8b4e4e` - updated the Phase 3 closeout review packet.
  - `59e37fa2` - normalized the reviewer feedback file layout.
- Recursive split/merge maintenance is deliberately guarded until recursive
  update propagation is designed and implemented.
- Deferred post-Phase-3 recursion items are carried forward in
  `plan/tasks/30-spire-ivf-foundation.md`:
  - durable per-level `nprobe` configuration/storage
  - durable per-level parameter storage
  - explicit user-facing per-level fanout configuration

## Repo Workflow

- At turn start, scan `review/` for new feedback and handle owned actionable
  feedback before new implementation.
- Work in narrow, testable slices.
- Commit each code/docs checkpoint and push immediately.
- Add/update the matching review packet in a separate commit and push.
- Do not run tests by default. For risky SPIRE/PostgreSQL behavior, prefer the
  narrowest PG18-focused validation.
- Do not run PG17 unless explicitly asked.
- Do not revert unrelated changes.

## Phase 4 Objective

Implement local multi-NVMe placement while staying on the local-node SPIRE
surface.

The durable shape is still:

```text
pid -> local_store_id -> object location
```

Phase 4 should move from the Phase 1/2/3 single relation-backed local store to
bounded partition-store relations that can each map to a PostgreSQL tablespace
expected to live on a physical NVMe device. The root/control index relation
remains authoritative. Do not make product claims about multi-NVMe performance
until benchmark-backed evidence exists.

## Read First

- `plan/tasks/30-spire-ivf-foundation.md`, especially Phase 4 and the Phase 3
  closeout follow-ups.
- `plan/design/spire-phase0-partition-object-storage.md`
- `plan/design/spire-recursive-hierarchy.md`
- `plan/design/spire-update-mechanics.md`
- `docs/SPIRE_DIAGNOSTICS.md`
- `src/am/ec_spire/{build,storage,meta,scan,update}.rs`

## Phase 4 Checklist

- Partition-store relation layout: define bounded store relations and how each
  maps to a PostgreSQL tablespace expected to live on a physical NVMe device.
- Hash placement: place leaf and internal partition objects by
  `hash(pid) % local_store_count`.
- Parallel local fetch: fetch selected PIDs grouped by local store and keep
  scoring close to partition object bytes.
- Placement diagnostics: expose per-store object count, bytes, candidate rows,
  and scanned PID counts. Existing single-store placement diagnostics are only a
  starting point.
- Local placement benchmark: measure one-store versus multi-store behavior on a
  machine with multiple physical NVMe devices before any product claim.

## Suggested First Slice

Start with a Phase 4 design checkpoint before code.

Define:

- the bounded store count/configuration surface
- how store relations are named, created, opened, and discovered
- how each store maps to a tablespace
- where root/control metadata records the active store set
- how single-store indexes continue to work without migration surprises
- how placement entries represent `local_store_id` and object location
- lock ordering and publish atomicity across multiple store relations
- failure/degraded semantics when one local store is unavailable
- which diagnostics become authoritative for placement state

Recommended artifact:

- Add `plan/design/spire-local-multistore-placement.md`.
- Add a review packet such as
  `review/30509-spire-phase4-local-placement-design/`.
- Commit the design checkpoint, push, then commit the review packet and push.

## Follow-On Slices

After the design checkpoint, proceed in narrow implementation slices:

1. Add local store configuration metadata and diagnostics.
2. Add store relation create/open helpers with default single-store behavior.
3. Add deterministic hash placement planning for leaf and internal PIDs.
4. Route object writes by `local_store_id`.
5. Group scan reads by local store before candidate scoring.
6. Extend placement diagnostics for multi-store placement and query touches.
7. Add benchmark harness and measurement packet only after correctness is
   stable.

Keep Phase 4 conservative: preserve the current single-store behavior and
diagnostics while adding multi-store placement as an extension.
