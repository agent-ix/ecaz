---
task: 198
packet: 005-full-scale-decision
head_sha: c6cabc917
extension_sha: 2ff72b3e49609c44cec881f72edf183a83554412
role: coder
date: 2026-07-23
status: open
---

# Review request: full-scale decision

Please review the complete decision evidence in
[`artifacts/manifest.md`](artifacts/manifest.md).

Task 198 recommends **PROMOTE to Task 199 productionization**, not direct
default promotion. The faithful replica preserves exact owner-arm recall at
0.9990 / 0.9685 / 0.9625 and improves warm mean by 15.9% / 14.0% / 17.0% at
10k / 50k / 100k. P95 and reconciled traversal improve at every scale, with
remote traversal eliminated and final payload reads still owner-side.

The accepted envelope must remain explicit: the replica is about 65–66% of
the physical generation (1.660 GB at 100k), copies 1.315 GB, emits
1.752–1.926 GB WAL across independent 100k builds, and takes about 51–52
seconds to build. Peak streaming batch memory is 3.367 MB.

The final corrected lifecycle runs prove a genuine failure after one completed
replica expansion at every scale (`fallback_count=1`), full owner restart,
corrupt-image fallback, exactly one `40001` invalidation attempt, retry through
owners, and idempotent retire/reclaim. The original packet-004 false line is
retained as audit history and explicitly superseded by the packet-005 100k
lifecycle suite.

Task 199 is checked in as the separate production gate. Until it passes,
normal production behavior remains owner traversal and Task 198's scan
selection stays feature-gated.
