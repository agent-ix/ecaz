# Review Request: SPIRE Production Heap Handoff Contract

- code commit: `f92cec900447ce71cf15a1ae09502fb09ebfb392`
- reviewer focus: packet 30728 P3 design-doc follow-up before C5
- phase: Phase 11 Stage C/D boundary

## Summary

This documentation checkpoint records two contracts before AM scan integration:

- future production executor stages should follow the same monotonic
  pending/sent/ready/failed state-extension pattern used by transport and
  candidate receive;
- `CandidateReceiveReady` dispatches are the only compact-candidate handoff
  into Stage D remote heap resolution.

The C5 section now states that AM scan integration may prove ordered compact
candidate merge, but final SQL rows must continue to surface
`requires_remote_heap_resolution` until Stage D resolves origin-node heap
visibility.

## Validation

- `git diff 3c42e2acc875ce2215761ebb3e7ab0a26df5ee90 f92cec900447ce71cf15a1ae09502fb09ebfb392 --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass.

No code paths changed in this checkpoint.

## Requested Review

Please check whether this is enough design guidance for the next C5 slice:

- Does the stage-extension pattern leave enough room for C2 cancellation and C4
  strict/degraded states?
- Is the `CandidateReceiveReady` to Stage D handoff specific enough to avoid
  duplicated compact-receive bookkeeping in AM scan integration?
