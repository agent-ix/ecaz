---
reviewer: opus47
status: open
created: 2026-05-01
checkpoint_commit: 12536e87
verdict: changes-requested
---

# Review: SPIRE Partition Object Design (packet 30161)

Scope of review: ADR-049, plan/tasks/30, US-017..US-020, FR-038..FR-043,
spec.md, spec/adr/index.md, plan/tasks/README.md.

The packet is internally coherent and the partition-object framing is a clear
improvement over the earlier `(vec_id, partition_id)` column note. The blockers
below are about traceability and a few AC-level issues, not the design shape.

## Blockers (must fix before status promotion)

### B1. `spec/tests.md` has zero coverage for any of the new artefacts

`spec/tests.md` is unchanged. None of US-017..US-020 or FR-038..FR-043 appear in:

- the Stakeholder Requirement Coverage table (StR-005 row is unchanged)
- the User Story Coverage table
- the Functional Requirement Coverage table
- the Test Case Summary
- the Coverage Gaps table

This violates Rule 1 of the matrix ("every acceptance criterion should trace to
at least one test case or documented gap"). DiskANN's FR-034..FR-036 set the
precedent of adding TC-010..TC-012 alongside the FRs; SPIRE was not given the
same treatment.

Minimum acceptable fix:

- Extend the StR-005 row's "Trace to US/FR/NFR" to include US-017..US-020 and
  FR-038..FR-043.
- Add US-017..US-020 rows to the User Story Coverage table.
- Add FR-038..FR-043 rows to the Functional Requirement Coverage table.
- Add at least one TC stub per FR (e.g. `TC-021..TC-026`, status `Planned`,
  pointing at the Phase 0 design packet) OR a `GAP-007` entry that explicitly
  records "SPIRE FR/US coverage deferred until Phase 0 design packet".

Without this, the test matrix silently drops six FRs and four US.

### B2. `StR-005` was not extended to cover SPIRE

`spec/stakeholder/StR-005-multi-am-vector-search.md`:

- Success criteria 1–3 mention only HNSW/IVF/DiskANN; SPIRE is absent.
- The `relationships` block lists only US-012, US-013, US-014; US-017..US-020
  declare `derives_from: StR-005` but the parent does not declare them as
  children. Bidirectional traceability is broken.

Either:

- Add US-017..US-020 to StR-005's `relationships` and add a fourth success
  criterion describing the SPIRE planning surface (PID-addressed partition
  objects, configurable consistency, future multi-machine path); or
- Introduce a new StR (e.g. `StR-007 — Recursive Billion-Scale ANN`) and
  re-point US-017..US-020 at it. Given that StR-005 is already approved and
  scoped to AM portfolio, extending it is the lighter touch.

Either way, the inconsistency between the four new US and StR-005 must be
resolved before any of the SPIRE US can move past DRAFT.

### B3. `FR-043-AC-1` is a documentation AC, not a behavior AC

> The first local implementation documents whether inserts/deletes use live
> deltas, mutable partition objects, or replacement epochs.

This is verifiable only by reading docs, not by test or measurement. Two
options:

- Move the documentation expectation into the Phase 0 design-packet deliverable
  in `plan/tasks/30-spire-ivf-foundation.md` (it already calls for an
  epoch/version note), and rewrite FR-043-AC-1 as a behavior AC such as
  "inserts and deletes against an active strict-mode epoch SHALL either be
  visible to a subsequent search at a published epoch or fail explicitly".
- Or keep the documentation requirement, but mark it as `evidence: design-note`
  in the AC and have `tests.md` route it to a docs-audit TC rather than a code
  test.

As written, FR-043 is the only SPIRE FR where the first AC cannot fail in a
test run.

## Significant gaps (should fix before implementation starts)

### S1. No failure-path ACs for split/merge or epoch publish

- US-020 covers happy-path publish, retain, and inspect, but has no AC for
  "publish fails partway" (e.g. a partition object is durable but the manifest
  write crashes). FR-041 also stops at "old epochs remain readable"; there is
  no AC for "abandoned/half-published epoch SHALL NOT become active and SHALL
  be reclaimable".
- FR-043 split/merge ACs cover correctness during a successful transition
  (AC-2) and cleanup (AC-3) but not abort/rollback. Phase 2 of the task plan
  mentions "concurrency validation" but the FR itself has no failure AC.

Add at least one AC each to US-020 and FR-043 for the abort path. A single
"failed epoch publish does not poison the active epoch and is recoverable
through diagnostics" AC covers both.

### S2. Heap-TID stability is unstated

`FR-038` ASSIGNMENT_ROW carries `tid heap_tid`. PostgreSQL UPDATE moves rows
(new CTID) and HOT chains complicate this further. None of FR-038, FR-040, or
FR-043 address how SPIRE handles heap_tid invalidation between publication and
read. Two routes:

- State that SPIRE only persists `vec_id` and resolves `heap_tid` at search
  time via index-on-vec_id (extra cost), or
- Accept stored `heap_tid` and require an explicit vacuum/repair path when
  HOT/UPDATE invalidates it (parallel to the DiskANN repair path).

Phase 0 design note in plan/tasks/30 should explicitly call out HOT/UPDATE
behavior; right now it is implicit.

### S3. `vec_id` width and uniqueness are unconstrained

ADR-049 says "may be derived from or mirror the heap TID" and FR-038 schema
declares `vec_id bytea`. There is no upper bound, no uniqueness scope (per
index? per heap?), and no statement of how local-only vec_id rebases to global
vec_id when multi-machine arrives. This will become contentious when
distributed merge by `vec_id` is being implemented.

Add to FR-038 (or a new note in the Phase 0 deliverable):

- Maximum encoded width.
- Uniqueness scope: per index OID is the obvious answer.
- Migration story when local vec_id (heap-TID-derived) is replaced by a global
  ID — does the format reserve a discriminator byte, or is rewrite required?

### S4. `FR-040-AC-1` bundles boundary-replication dedup into single-level

> Single-level SPIRE can route to leaf PIDs, score candidates, dedupe by
> `vec_id`, and return ordered local heap TIDs.

In the single-level Phase 1 there is one `(vec_id, pid)` row per vector and
nothing to dedupe. Boundary replication is Phase 5 of the task plan. As
written, the single-level implementation must implement dedup it does not need
yet — or the AC is met trivially.

Split it:

- FR-040-AC-1 (single-level): route + score + return ordered heap TIDs.
- FR-040-AC-3 (boundary-replication): dedup by `vec_id` and report suppressed
  duplicates.

The current AC-3 already covers the latter; AC-1 should drop "dedupe by
vec_id" so the Phase 1 contract matches Phase 1 work.

### S5. Default consistency mode for the local single-store path is unspecified

FR-041 §5 says graceful degradation is "preferred for large remote deployments"
and exposes both modes, but never names the v1 local-single-store default.
Without this, the single-level Phase 1 ec_spire will have to invent a default
silently. State it in FR-041 (probably `strict` for local single-store, since
"unavailable store" with one store means index unusable).

## Minor

- `FR-038` declares `relationships: implements US-017` only. FR-038 is also a
  load-bearing dependency for FR-040, FR-041, FR-043 (they all reference its
  schema). Either add inter-FR `depends_on` relationships, or accept that the
  cross-FR linkage lives only in prose. Existing FR set is inconsistent on
  this; the SPIRE cluster is a good moment to pick a convention.
- `ADR-049` Status block says "Proposed"; the index.md row says "PROPOSED".
  The ADR's `impact:` line says "Affects ADR-035, ADR-048" — ADR-035 is
  DROPPED in the index, so that reference is misleading. Either drop ADR-035
  from the impact line or note that ADR-049 supersedes the SPANN direction
  in the prose.
- `plan/tasks/30-spire-ivf-foundation.md` Phase 0 lists six design notes plus
  a review packet, but does not call out HOT/UPDATE handling for `heap_tid`
  (see S2). Add as a seventh bullet.
- `FR-042` `spire_remote_search_request` schema includes `consistency_mode`
  but the AC set (FR-042-AC-1..3) never exercises strict-vs-degraded across
  remote nodes. FR-041-AC-3 covers degraded reporting in general, but the
  remote-node fail-closed path deserves its own AC under FR-042 since it is
  the only place strict mode crosses a process boundary.
- `spec.md` §3.2 row for `ec_spire` says `Opclasses: TBD`. That is fine for
  draft, but the docs gate in plan/tasks/30 Phase 8 ("Update README/user
  docs") should be prerequisite on filling this in. Consider an explicit
  Phase 1 deliverable: "register opclass names in spec.md".
- `FR-039-AC-1` is "at least one local store can be configured through SQL"
  — that is satisfied by the single-store default already required by
  FR-038 #6. AC-2 is the real multi-store gate. Consider folding AC-1 into
  AC-2's preamble, or reword AC-1 as "the single-store path SHALL expose the
  same configuration/diagnostics surface that multi-store will use" so the
  AC has independent value.

## Things explicitly OK

- Naming and numbering: US-017..US-020, FR-038..FR-043, ADR-049 are all the
  next available IDs; no collisions with the duplicate-ADR-number history.
- Bidirectional links US <-> FR are consistent; only the StR<-US link is
  broken (B2).
- Mermaid diagrams in FR-038/FR-039/FR-040/FR-041/FR-042/FR-043 render and
  match the prose. The ER diagram in FR-038 captures the right entities.
- ADR-049 honors the "no speculative pluggable abstractions" feedback in
  Decision 5 explicitly.
- Phase staging in plan/tasks/30 puts the highest-risk decision (storage
  shape, Phase 0) first and defers boundary replication to Phase 5, which
  matches the ADR risk discussion.
- Out-of-Scope section in plan/tasks/30 and spec.md §2.2 both list the same
  deferrals (product billion-scale claims, GPU/offline trainer, declarative
  table partitions). No drift between the two.

## Suggested order of operations

1. Fix B1 (tests.md) and B2 (StR-005) together — these are the only blockers
   for marking US-017..US-020 and FR-038..FR-043 as APPROVED.
2. Fix B3 (FR-043-AC-1) and S4 (FR-040-AC-1 split) — small AC edits.
3. Add Phase 0 design-note bullets for HOT/UPDATE (S2) and `vec_id` width/
   uniqueness (S3) so the implementer cannot avoid the decision.
4. Address S1 and S5 in the next FR pass.
5. Minor items can land alongside the Phase 0 design packet.

The packet is close. With B1–B3 cleared and Phase 0's design note drafted
against S2/S3, it is ready for implementation planning.
