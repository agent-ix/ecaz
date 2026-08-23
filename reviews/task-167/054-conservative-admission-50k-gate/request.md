---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 conservative-admission 50k gate

Status: measured negative; the conservative-admission candidate is rejected.
No Task 167 acceptance or closeout is claimed.

Packet 053 introduces a conservative free-capacity backlink policy. It screens
the existing-neighbor-plus-backlink union with exact-distance `robust_prune`,
but mutates the target only when the new backlink and every existing edge
survive. Full targets retain ordinary re-pruning.

This packet freezes one clean isolated 50k branch-point run before installing
or measuring the exact runtime. It keeps packet 051's operating point: the same
160 insert sources, 200 heldout queries, 48 inserted-neighborhood queries,
exact fp32 truth, graph/search settings, and packet-045 hard bands. The
unconditional-append diagnostic control cannot mutate the fixture before the
candidate quality gate passes.

The thresholds will not be widened after observing the run. Failure rejects
this candidate and leaves Task 167 open. A pass permits, but does not replace,
the required isolated 10k/50k/100k recall, latency, and storage matrix.

The exact-runtime run failed the heldout hard gate after 160 isolated
conservative-admission inserts. The inserted-neighborhood population improved
materially and passed: physical `0.923735` versus fresh `0.931052`, a
`0.007316` deficit against the fixed `0.015000` band. The dominant 200-query
heldout population measured physical `0.847722` versus fresh `0.857333`, a
`0.009611` deficit against the fixed `0.007000` band: a miss of `0.002611`.
The unconditional-append control and all post-gate drills were skipped before
they could mutate the fixture.

The candidate rejected 133 of 4,987 attempted free-capacity backlink
amendments. Its heldout deficit was `0.001000` better than packet 051's
append-only result but `0.001000` worse than packet 047's robust-prune-all
result. It therefore does not clear the 50k branch point, and the final scale
matrix remains blocked on another isolated candidate.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
