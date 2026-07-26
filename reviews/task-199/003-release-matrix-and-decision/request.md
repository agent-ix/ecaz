---
task: 199
packet: 003-release-matrix-and-decision
agent: coder
role: coder
model: gpt-5
date: 2026-07-25
---

# Task 199 release matrix and decision

Please review the productionization fix at `2a4a70b23` (including `0c4bf83f0`).
The fix preserves fresh-snapshot replica invalidation while caching the
known-absent path, and avoids stale statement-snapshot negative caching.

The exact release A/B matrix completed at 10k, 50k, and 100k. Recall is
unchanged between owner and traversal-replica arms at each scale; replica
latency is 16.00, 16.40, and 16.20 ms versus owner 18.30, 19.40, and 19.90
ms. No-replica mutation throughput is 2292.764, 2564.925, and 2524.220
rows/s. Storage is identical between arms at every scale. All physical
topology and materialization gates passed; see `artifacts/manifest.md` and
the packet-local summaries.

The historical pre-task baseline at 10k was 2315.234 rows/s, so the final
10k matrix is within 1% (the independent lifecycle run measured 2481.671
rows/s). Task 198 owner-control latency references remain above the final
owner values at 10k/50k/100k. The Graviton evidence proves ordered identity
and lifecycle correctness within aarch64 (Graviton4/Neoverse V2, SVE2-128);
it does not claim cross-ISA comparison of one shared generation. That
comparison is explicitly waived for this task and deferred to a shared-
generation portability task.

Decision: promote pending outside reviewer acceptance of this packet and the
already refreshed lifecycle packet 002.
