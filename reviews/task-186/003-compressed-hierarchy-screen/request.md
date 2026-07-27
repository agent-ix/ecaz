---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 186 compressed/hierarchy screen decision

This is a historical hierarchy checkpoint only. It is not a compressed-head
screen and does not support a family-wide rejection.

The required hierarchy screen is recorded in packet
[`002-hierarchy-screen`](../002-hierarchy-screen/request.md). This packet is
the task-local candidate-family decision checkpoint: the measured
two-level/representative route is rejected because it achieved 0.9440 recall
at 84.30 ms mean while the exact 16,384 control achieved 0.9740 recall at
27.10 ms mean. Coverage was not the failure mode (zero fraction 0 and owner
membership 0.6155).

No separate compressed-head implementation was performed in this checkpoint.
The retained head capacity is an evidence-only control, and any persisted
format or routing change would require a separate production task and ADR.
The hierarchy prototype's query-time region rebuild, arbitrary representatives,
hard-coded 256/16/512 caps, and absent routing/build counters limit the STOP to
that prototype. A build-time-assigned hierarchy and compressed head remain
unscreened alternatives.

See the packet-local [manifest](../002-hierarchy-screen/artifacts/manifest.md)
and structured [results](../002-hierarchy-screen/artifacts/run/results.jsonl)
for the immutable measurement evidence.
