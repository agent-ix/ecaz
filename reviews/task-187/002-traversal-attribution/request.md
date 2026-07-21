---
task: 187
packet: 002-traversal-attribution
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Traversal attribution and candidate gate

The fresh 100k run attributes 7.468 ms of the 22.40 ms warm mean to
traversal. Remote owner expansion is 6.174 ms (82.7% of traversal), local
expansion 1.230 ms, and the derived coordinator/frontier remainder 0.065 ms.
Head scoring and seed selection are 2.145 ms and 0.094 ms respectively.

The existing counters also quantify the likely transport share: remote
expansion is 493 samples (about 9.9 per query, 0.626 ms each) versus 398 local
expansions (about 8.0 per query, 0.155 ms each). Using local expansion as an
owner-work proxy leaves roughly 0.47 ms per remote expansion, or about 4.6 ms
per query, as transport/encoding/wait overhead. Together with the 9.991 ms
materialization request-wait component, roughly 16 ms of the 22.40 ms warm
mean is remote request waiting.

This rules out frontier insertion, hop bookkeeping, and head scoring as useful
first candidates. The warm path already uses pooled owner sessions and
concurrent per-hop requests; the measured connection-ready component is
0.002 ms in the materialization path. No isolated cache, packing, hop-fusion,
or straggler change has a demonstrated causal target, so no production
candidate is advanced. Task 187 therefore takes the contract's STOP branch;
the next task should instrument or redesign the remote owner transport before
attempting another end-to-end optimization.

Evidence: packet 001 summary, `results.jsonl`, and `suite-manifest.json`.
