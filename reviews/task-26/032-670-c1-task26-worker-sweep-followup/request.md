# Review Request: Task 26 Worker Sweep Follow-Up

## Summary

Please review commit `5054875`, which updates Task 26 with the packet 668
real-50k worker sweep result and clarifies the next Phase 5 scale target.

This is a docs-only checkpoint. It does not change runtime behavior.

## Changes

- Records the real-50k worker sweep timings from packet 668 directly in Task 26.
- Notes that 4 workers is the current best point on this PG18 cluster.
- Notes that the 8-worker surface needs enough `max_worker_processes` headroom
  before drawing a scaling conclusion, because the 8-request run launched only
  7 graph workers.
- Sets the next scale target to the DBPedia 990k/10k profile once chunked
  prepare/load support from Task 10066 is available, or after an accepted
  one-shot non-resumable load.

## Validation

- `git diff --check`

## Notes

No new raw measurement is attached. Packet 668 remains the source of truth for
the worker sweep logs.
