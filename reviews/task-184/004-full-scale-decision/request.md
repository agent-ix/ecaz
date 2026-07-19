---
task: 184
packet: 004-full-scale-decision
role: coder
status: open
date: 2026-07-19
head: 765f28a54
---

# Review request: full-scale materialization decision

Packet 003 qualified the single preregistered batch-10 candidate with identical
recall and semantics plus a 40.5% 100k mean-latency reduction and materially
better tails. This packet runs the required checked-in eager/lazy A/B at 10k,
50k, and 100k before deciding whether the candidate should advance from its
benchmark-only implementation to a production follow-up.

Every scale uses the retained production head policy, persisted-head seed mode,
32 seed search/return width, BW4/H100, RaBitQ stored neighbor codes and traversal
scoring, exact final ranking, three physical owners, 200 held-out queries / 2,000
distinct top-10 trials, and 50 warm latency samples after 10 warmups at
concurrency one. The two variants share one immutable generation per scale and
must attest the same seed digest. Stage/work attribution, storage, construction,
topology, engagement, query separation, and unanimous installed release
provenance are required at every scale.

The request remains open until all three steps complete and the packet records
a promote/iterate/stop decision using relative Pareto evidence.
