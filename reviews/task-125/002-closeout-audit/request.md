---
task: 125
topic: closeout-audit
requester: codex
date: 2026-07-01
code_commit: 371db1bdc
base_commit: 6799686af9e9adf13332bd4ec6e19b60e7ceb80e
---

# Review Request: Task 125-129 Closeout Audit

This packet audits the current `task-125-tq-scorer-optimization` branch against
the Task 125-129 definitions found on `origin/task-124-ivf-tq-stage2`.

Result: do not close the literal 125-129 objective yet. The branch contains a
real, reviewed TurboQuant scorer optimization, but the current evidence does
not prove every explicit task requirement.

Key findings:

- Task 125 is partially satisfied: the accepted implementation shrinks the
  no-QJL 4-bit LUT to `i16 + scale`, with recall-safe 10k/50k/100k evidence.
  The requested cross-block dimension cache-blocking is not implemented: the
  current dispatch still walks candidate blocks through `score_width_cascade`
  and calls `score_lut_no_qjl_4bit_block32` per block.
- Task 126 is partially satisfied: `BLOCK_WIDTH` is raised to 64 and covered by
  correctness plus suite evidence, but there is no committed 32/64/128
  per-width curve.
- Task 127 is partially satisfied: NEON suffix-bound pruning exists and recall
  is unchanged, but the committed packet does not report a pruned fraction.
- Task 128 is satisfied for the TurboQuant no-QJL scorer path covered here:
  caller-owned score buffers are reused and the no-QJL negated scoring path
  writes directly to caller output.
- Task 129 is satisfied for the TurboQuant no-QJL scorer path covered here:
  no-QJL batch scoring avoids gamma side input and redundant code-slice
  rebuilding in the hot candidate-batch path.

The detailed matrix is in `artifacts/closeout-audit.md`.

Requested review:

- Confirm this audit correctly distinguishes accepted optimization evidence
  from literal closeout requirements.
- If accepted, continue with a follow-up implementation packet for the missing
  Task 125 cross-block dimension cache-blocking and Task 126 128-wide sweep
  before marking 125-129 complete.
