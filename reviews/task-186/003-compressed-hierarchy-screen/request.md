---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 186 compressed/hierarchy screen decision

The required hierarchy screen is recorded in packet
[`002-hierarchy-screen`](../002-hierarchy-screen/request.md). This packet is
the task-local candidate-family decision checkpoint: the measured
two-level/representative route is rejected because it achieved 0.9440 recall
at 84.30 ms mean while the exact 16,384 control achieved 0.9740 recall at
27.10 ms mean. Coverage was not the failure mode (zero fraction 0 and owner
membership 0.6155).

No separate compressed-head implementation is advanced. The retained head
capacity is an evidence-only control, and any persisted format or routing
change would require a separate production task and ADR.

See the packet-local [manifest](../002-hierarchy-screen/artifacts/manifest.md)
and structured [results](../002-hierarchy-screen/artifacts/run/results.jsonl)
for the immutable measurement evidence.
