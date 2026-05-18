# Review Request: Task 41 Invariant #2 Memory Lifetime Strategy

Base survey head: `7fef354f16ebd44eca63700ff666ee4b479ac189`

## Summary

This packet steps back from individual local refactors and lays out a full
strategy for completing Task 41 invariant #2.

The strategy is deliberately two-level:

- high-level model: classify every PostgreSQL-memory lifetime surface by
  ownership/lifetime mechanism;
- ground-level sequence: local, reviewable packets that finish the work without
  colliding with the other Task 41 invariant tracks.

## Scope

No code changes in this packet. The strategy covers:

- Datum / varlena memory;
- tuple-slot Datums;
- palloc-backed scan-state arrays;
- buffer/page byte views;
- PostgreSQL-owned C strings.

## Artifacts

- `artifacts/strategy.md`
- `artifacts/detoast-inventory.log`
- `artifacts/raw-slice-inventory.log`
- `artifacts/slot-datum-inventory.log`
- `artifacts/palloc-inventory.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm the high-level model covers all memory-context lifetime surfaces in
  Task 41 invariant #2.
- Confirm the ground-level packet order is local enough for parallel work and
  avoids overlap with invariant #1 and invariant #3.
- Confirm the stop conditions are concrete enough to decide when invariant #2
  is actually complete.
