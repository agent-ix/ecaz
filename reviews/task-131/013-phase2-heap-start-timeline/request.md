# Review Request: Phase 2 Heap Start Timeline

- task: 131
- packet: `reviews/task-131/013-phase2-heap-start-timeline/`
- code commit under review: `d4c2e324658f0f4b52196343e1c00d2d282e5b88`
- predecessor packet: `reviews/task-131/012-production-scan-profile-mi-smoke/`

## Context

Task 131 Phase 2 requires evidence that candidate-to-heap streaming removes the candidate-phase barrier where safe. The production session-reuse path already runs heap receive on each candidate session, but the timeline was too weak to prove ordering because `heap_receive.started_after_ms` was copied from the candidate request start.

## Changes

- Changed the session-reuse heap receive path so `SpireRemoteProductionHeapReceiveResult.started_after_ms` records the actual heap query start.
- Changed heap `elapsed_ms` in that path to measure heap query duration rather than the full candidate session duration.
- For global-preheap empty-subset rows, report a zero-duration heap row at the no-op completion timestamp.

## Evidence

See `artifacts/manifest.md`.

The local four-instance PG18 slow-node fixture now reports:

- candidate receives complete at 12-13 ms across nodes 2, 3, and 4;
- heap requests start at 13 ms;
- fast-node heap receives complete at 25 ms;
- slow node-2 heap receive completes at 624 ms.

That is direct timeline evidence that fast heap work begins and completes before the slow-node heap path finishes. This packet is an instrumentation/evidence checkpoint for Phase 2, not a full Phase 2 closeout matrix.

## Validation

- `cargo check --lib` passed.
- `scripts/run_spire_phase13e_static_remote_placement_pg18.sh` passed against a local four-instance PG18 fixture.

## Reviewer Notes

- This does not yet claim Task 131 Phase 2 is complete. Phase 2 still needs broader skewed/normal fixture timing evidence, strict/degraded correctness coverage at the intended matrix scale, and a decision on whether additional code changes are needed beyond the existing session-reuse path.
