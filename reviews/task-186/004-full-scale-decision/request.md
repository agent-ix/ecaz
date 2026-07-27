---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 186 full-scale decision

This packet is a disposition of the historical screen, not a family-wide
closeout. The entry correction is recorded in
[`005-entry-and-head-design`](../005-entry-and-head-design/request.md), which
cites Task 185's outside-reviewed STOP and preserves its qualifications.

## Decision

STOP for the tested hierarchy prototype. No Task 186 candidate met the
promotion rule. The exact capacity screen showed a monotonic recall signal
through 16,384, but its cost tradeoff was not Pareto-safe; the bounded
hierarchy then lost recall and latency despite zero coverage misses. This does
not reject a build-time-assigned hierarchy or the compressed-head alternative:
the historical run did not screen the compressed arm and did not produce the
routing/build counters needed for a bounded-work proof. No production task is
opened from this packet.

## Evidence

- [capacity-control packet](../001-capacity-control/request.md)
- [hierarchy-screen packet](../002-hierarchy-screen/request.md)
- [candidate-family checkpoint](../003-compressed-hierarchy-screen/request.md)

All cited runs passed topology, remote-owner engagement, and provenance gates.
The STOP is evidence-based and prototype-scoped. The capacity rows are
independent generations, so their small recall deltas are not a paired
per-query comparison; mechanism coverage is the stronger observed monotonic
signal. A paired recall test was not available from the historical aggregate
artifacts and is required before any future capacity candidate is advanced.
