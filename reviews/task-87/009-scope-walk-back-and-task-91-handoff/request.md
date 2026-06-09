# Task 87 Packet 009: Scope Walk-back and Task 91 Handoff

## Summary

This packet responds to reviewer feedback on packet 007:

- `reviews/task-87/007-phase1-common-codec-scope-revision/feedback/2026-06-08-01-reviewer.md`

Task 87 is restored to its original premise: `CandidateBatch` and
per-AM batch-shaped scoring on existing per-AM codec surfaces. The
cross-AM `QuantCodec` migration, trait growth, and AM adapter migration
work now belong to Task 91.

Code and documentation checkpoint under review:

- `55eb1685f29c858a0cddbb0acc848ca62f519d1b` - prior head with Task 91 opened
- this packet's commit - scope walk-back in Task 87 / Task 90 plus packet-local review context

## Changes

- Updated `plan/tasks/87-candidate-batched-scoring-across-ams.md` to:
  - keep `CandidateBatch` and per-AM batched scoring as Task 87's goal;
  - keep batch-shaped scoring as an architectural requirement;
  - remove common quant codec shape and cross-AM codec migration from
    Task 87 acceptance;
  - reference Task 91 for `QuantCodec` migration, trait growth, AM
    adapter migrations, and DiskANN TurboQuant search-codec work;
  - restore DiskANN Phase 4 to Stop Condition territory, with packet
    005 again serving as the Task 87 DiskANN handoff.
- Updated `plan/tasks/90-diskann-turboquant-search-codec.md` from
  "absorbed by Task 87" to "superseded by Task 91". Task 90 closes by
  reference when Task 91 Phase 6 ships.
- Left `src/am/common/quant_codec.rs` and the packet 008 IVF adapter in
  tree as Task 91 beachhead code, per reviewer direction.

## Packet Status Notes

- Packet 005 (`reviews/task-87/005-phase4-diskann-stop-condition/`) is
  un-superseded and remains the accepted Task 87 DiskANN resolution.
- Packets 007 and 008 remain historical context. Their in-tree trait
  code stays, but the claim that Task 87 owns the broader common codec
  migration is walked back here.
- A SPIRE common-codec adapter commit exists on this branch from the
  paused work. This packet does not ask reviewers to count that work
  toward Task 87 acceptance; it should be treated as Task 91-owned
  historical context unless the reviewer asks for a revert or branch
  split.

## Validation

No tests run. This is a documentation and review-scope correction only;
no source behavior changed in this packet.

## Review Focus

- Confirm Task 87 scope is back to CandidateBatch/per-AM scoring work
  and no longer owns cross-AM `QuantCodec` migration.
- Confirm Task 90 now correctly points to Task 91 as the owner for
  DiskANN TurboQuant search-codec completion.
- Confirm packet 005 is un-superseded and DiskANN remains a Stop
  Condition handoff for Task 87.
