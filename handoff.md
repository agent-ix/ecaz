# Task 30 Phase 5 Handoff

You are continuing Task 30 SPIRE after Phase 4 has merged to `main`.

## Current State

- Phase 4 merge commit on `main`: `8172d7cc` (`Merge SPIRE Phase 4`).
- Source branch that landed Phase 4: `task30-spire-partition-object-spec`.
- Reviewer merge-readiness verdict: `review/30509-spire-phase4-local-placement-design/feedback/2026-05-06-06-reviewer.md`.
- Phase 4 is closed for local multi-store placement:
  - auxiliary local store relations are created and published;
  - mutation paths route through the active local store set;
  - scan prefetch resolves placements and batches PG18 `ReadStream` reads per
    local store relation;
  - SQL `VACUUM` has two-store coverage;
  - storage-debt diagnostics aggregate root/control plus auxiliary stores;
  - same-device and `/mnt/e` two-store benchmark evidence is packet-local;
  - multi-store REINDEX rejects explicitly until a full lifecycle exists;
  - auxiliary store autovacuum is disabled at the parsed relcache options
    boundary.
- `plan/status.md` has Task 30 at 94%. Remaining Task 30 gates are
  PQ-FastScan scorer binding and physical object reclamation / old-epoch
  cleanup. Those are not Phase 5 boundary-replication prerequisites unless the
  task explicitly scopes them in.

## Repo Workflow

- Start each turn by scanning `review/` for new feedback. Process owned,
  actionable feedback before new implementation work.
- Work in narrow, testable slices.
- Commit each code/docs checkpoint and push immediately.
- Add or update the matching review packet in a separate commit and push.
- Do not run tests by default. For risky SPIRE/PostgreSQL behavior, prefer the
  narrowest PG18-focused validation.
- Do not run PG17 unless explicitly asked.
- Do not revert unrelated changes.

## Phase 5 Objective

Implement boundary replication for SPIRE.

The durable storage shape already supports multiple assignment rows per vector:

```text
vec_id -> one or more pid assignments
pid -> local_store_id -> object location
```

Phase 5 should turn that latent shape into controlled behavior: a vector can be
assigned to its primary partition and one or more nearby boundary partitions,
then scans must deduplicate replicated `vec_id`s before final top-k output.

## Read First

- `plan/tasks/30-spire-ivf-foundation.md`, especially Phase 5 and the open
  PQ-FastScan / old-epoch cleanup notes.
- `plan/design/spire-phase0-partition-object-storage.md`
- `plan/design/spire-recursive-hierarchy.md`
- `plan/design/spire-update-mechanics.md`
- `plan/design/spire-local-multistore-placement.md`
- `docs/SPIRE_DIAGNOSTICS.md`
- `src/am/ec_spire/{assign,build,scan,update,storage,meta}.rs`
- Phase 4 review packets `30531` through `30540` for local-store behavior that
  boundary replication must preserve.

## Phase 5 Checklist

- Boundary predicate: define the threshold/rule for assigning a vector to
  multiple nearby partitions.
- Assignment fanout: extend the assignment writer from one row per vector to
  multiple `(vec_id, pid)` rows.
- Duplicate control: ensure scans deduplicate replicated vector IDs before
  final top-k.
- Recall study: measure recall delta with boundary replication off/on at fixed
  storage overhead.
- Storage accounting: report leaf-assignment and posting-list growth from
  replication.

## Suggested First Slice

Start with a design checkpoint before changing build behavior.

Define:

- the reloption/GUC surface for enabling boundary replication, including a
  conservative default-off path;
- the boundary predicate, such as top-N nearby leaves or distance-margin based
  fanout;
- hard caps for assignment fanout so storage growth is bounded;
- how the existing explicit scan dedupe mode transitions from primary-only to
  replicated-assignment mode;
- how diagnostics expose primary assignment count, boundary replica count,
  total assignment rows, and estimated overhead;
- how local multi-store placement remains hash-by-PID and does not need a new
  store placement rule.

Recommended artifact:

- Add `plan/design/spire-boundary-replication.md`.
- Add a review packet such as
  `review/30541-spire-boundary-replication-design/`.
- Commit the design checkpoint, push, then commit the review packet and push.

## Implementation Slices

1. Add parsed options and diagnostics for boundary replication while preserving
   default primary-only behavior.
2. Extend assignment planning to compute bounded secondary PIDs without writing
   them yet.
3. Publish multiple assignment rows per vector in a small PG18 fixture.
4. Switch scan dedupe to the replicated-assignment mode when fanout is enabled.
5. Add storage-accounting diagnostics for primary rows, replica rows, and
   growth ratio.
6. Run a small recall/storage comparison packet with replication off/on at a
   fixed target overhead.

## Guardrails

- Preserve existing single-store and multi-store Phase 4 behavior.
- Do not make product claims from local recall/latency data.
- Do not implement full multi-store REINDEX lifecycle in Phase 5 unless the
  user explicitly asks; it is recorded as later lifecycle work.
- Do not pull in repo-wide hardening items from the Phase 4 review deferral
  list unless explicitly scoped.
- Keep PQ-FastScan populated-index scorer binding separate unless the boundary
  replication change directly touches assignment payload scoring.
