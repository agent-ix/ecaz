---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 pruned-backlink no-op 50k gate

Status: measured negative; the pruned-backlink no-op candidate is rejected.
No Task 167 acceptance or closeout is claimed.

Packet 056 introduces a full-target-only insertion candidate. When exact
`robust_prune` excludes a proposed backlink, the target retains its exact
established adjacency and order instead of being rewritten without the new
reverse edge. Spare-capacity robust-prune behavior and incomplete-population
stale-neighbor cleanup are unchanged.

This packet freezes one clean isolated 50k branch-point run before installing
or measuring the exact runtime. It retains packet 054's operating point: the
same 160 insert sources, 200 heldout queries, 48 inserted-neighborhood queries,
exact fp32 truth, graph/search settings, and packet-045 hard bands. The
append-when-room control cannot mutate the fixture before the candidate quality
gate passes.

The thresholds will not be widened after observing the run. Failure rejects
this candidate and leaves Task 167 open. A pass permits, but does not replace,
the required isolated 10k/50k/100k recall, latency, and storage matrix.

The exact-runtime run failed the dominant heldout hard gate after 160 isolated
candidate inserts. The inserted-neighborhood population passed: physical
`0.922082` versus fresh `0.931052`, a `0.008970` deficit against the fixed
`0.015000` band. The 200-query heldout population measured physical
`0.847722` versus fresh `0.857333`, a `0.009611` deficit against the fixed
`0.007000` band: a miss of `0.002611`.

The candidate preserved 702 full targets whose prune rejected the proposed
backlink, but its heldout result was exactly equal to packet 054's rejected
conservative-admission result and `0.001000` worse than packet 047's retained
robust-prune result. The append control and all post-gate drills were skipped
before they could mutate the fixture. The candidate does not clear the 50k
branch point, and the final scale matrix remains unauthorized.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
