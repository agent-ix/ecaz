---
task: 118
packet: reviews/task-118/017-intel-closeout-handoff-refresh
checkpoint_sha: 812013aed951952550a5e884e3d950870cb0d4f1
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Intel Closeout Handoff Refresh

## Scope

This checkpoint refreshes the Task 118 Intel closeout handoff after packet 016
produced current-head 10k AMD diagnostics.

The important correction is that packet 016 is useful current-head preview
evidence, but it is not final host-class closeout evidence. The Intel handoff
now requires final Intel `ecaz bench suite` artifacts at all three required
Task 118 scales: 10k, 50k, and 100k.

Updated:

- `reviews/task-118/010-intel-closeout-runbook/artifacts/intel-closeout-runbook.md`
- `reviews/task-118/011-final-closeout-audit-template/artifacts/final-closeout-audit-template.md`

## Validation

- The added 10k command uses the same checked-in Task 118 suite config and the
  same artifact naming scheme as the existing 50k/100k Intel commands.
- The audit template now checks for 10k/50k/100k Intel manifests, result JSONL,
  and suite logs.
- 10k full-suite dry-run selected `36` steps: `6` each of load, recall,
  `hnsw-frontier`, `hnsw-score-correlation`, latency, and storage.
  - Artifact: `artifacts/suite-dry-run-10k-intel-shape.log`
  - Artifact: `artifacts/suite-manifest-dry-run-10k-intel-shape.json`
- Expected selected-step count now covers three scales: `108` selected steps
  total, `18` per selected step kind.

No benchmark was run in this packet. This is an operator handoff correction so
the final Intel pass satisfies the full Task 118 10k/50k/100k evidence gate.

## Remaining Task 118 Closeout Work

Run the 10k, 50k, and 100k suites on the Intel benchmark host, commit the final
packet 006 artifacts, and update packet 006 with the dominant recall-loss
classification table.
