# Task 123 Phase B Boundary-2 Status Sync Manifest

- Head SHA: `dc7b630a0878cdeb1d86b452391d41af9e8c0d3b`
- Task bucket: `reviews/task-123/007-phase-b-b2-status-sync`
- Evidence source: `reviews/task-123/006-phase-b-100k-n1024-b2-followup/`
- Status update commit: `dc7b630a0 Update task 123 b2 follow-up status`
- Timestamp: `2026-06-27T17:04:19Z`

## Files Updated

- `plan/tasks/123-spire-route-precision-scan-cost.md`
- `plan/tasks/README.md`

## Summary

The canonical task status now says **Phase B boundary-2 follow-up closeout
requested** and points at packet
`006-phase-b-100k-n1024-b2-followup`. It records that:

- Phase A high-recall `n128` missed the flat-floor latency gate.
- The 100k `nlists=1024` spot-check now covers boundary 0, 1, and 2.
- Boundary 2 improved recall but still reached only `309 / 320 = 0.9656` at
  nprobe 64, with clean p50 `526.0 ms` and a `246.0 MiB` SPIRE index.
- Task 123 remains a review-requested no-go / re-scope result and is not marked
  complete until outside reviewer sign-off.

No new benchmark was run for this status-sync packet; measurement evidence
remains packet 006.
