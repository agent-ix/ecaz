# Task 87 Packet 007: Common Codec and Broad Batch Scope Revision

## Summary

This packet responds to reviewer addenda 02 and 03 on packet 001. It
updates the authoritative Task 87 scope and adds a design addendum for
the common quant codec shape.

Files under review:

- `plan/tasks/87-candidate-batched-scoring-across-ams.md`
- `plan/tasks/90-diskann-turboquant-search-codec.md`
- `reviews/task-87/007-phase1-common-codec-scope-revision/artifacts/design-addendum.md`

## Changes

- Removed the stale "Task 87 is TurboQuant no-QJL 4-bit only" wording.
- Added the common quant codec shape as an explicit Task 87 shared
  surface alongside `CandidateBatch`.
- Added routing guidance that every batch-shaped quant mode should route
  through `CandidateBatch`, with non-routed cells requiring structural or
  measured justification.
- Revised per-AM phase text for SPIRE, IVF, DiskANN, and HNSW to cover
  broad quant routing instead of leaving other quant paths inline by
  default.
- Marked Task 90 as absorbed by Task 87's broadened scope, so it is no
  longer a preferred DiskANN deferral.
- Superseded packet 005 as a final DiskANN Stop Condition while keeping
  it as source evidence for the missing current TurboQuant codec.

## Validation

No tests were run. This is a documentation-only design/scope packet.

## Review Focus

- Confirm the routing table satisfies the addendum-02 requirement for
  per-(AM x quant mode) decisions and justifications.
- Confirm the common quant codec shape satisfies addendum-03 B6 at the
  design level.
- Confirm Task 90 is now correctly treated as historical context rather
  than a Task 87 DiskANN deferral.
- Confirm existing packets 003, 004, and 006 can remain as partial
  structural slices under the broadened scope, while packet 005 is
  superseded for closeout purposes.
