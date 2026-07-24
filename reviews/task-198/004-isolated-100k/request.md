---
task: 198
packet: 004-isolated-100k
head_sha: c7a2900fb
extension_sha: 2ff72b3e49609c44cec881f72edf183a83554412
role: coder
date: 2026-07-23
status: open
---

# Review request: isolated 100k coordinator-replica A/B

Please review the causal 100k A/B and its decision in
[`artifacts/manifest.md`](artifacts/manifest.md).

The faithful coordinator replica preserves the owner arm's `0.9625` recall
and ordered seed identity while improving warm mean latency from 20.60 to
17.10 ms, p95 from 23.80 to 20.00 ms, and p99 from 24.70 to 20.40 ms.
Attribution reconciles the movement: remote traversal is replaced by
coordinator-local graph/vector reads, reducing traversal from 7.866 to 3.617
ms, while owner-side final payload cost remains.

The cost is also decision-relevant: a 100k replica occupies 1.660 GB, emits
1.926 GB WAL, copies 1.315 GB, and takes 52.0 seconds to build. This packet
therefore advances the candidate to the complete matrix without promoting it
as a production default.

The packet also records a scale-specific harness gap: the fixed fault query
did not reach the injected second batch. `bb922fb9a` makes the drill
deterministically find and report a query that does, preserving owner-result
identity throughout. Packet 005 owns the corrected completion run.
