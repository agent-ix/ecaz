---
task: 190
packet: 002-architecture-comparison
role: coder
date: 2026-07-23
status: review_requested
---

# Review request: two-family architecture comparison

The comparison is deliberately limited to:

1. a fingerprint-bound coordinator traversal replica; and
2. dedicated binary traversal transport.

The replica is selected. It can remove the measured ten-round transport wait;
binary serialization cannot, and the measured connection/encode/decode share
is only 0.071 ms/scan. The trade is substantial derived storage and lifecycle
work: up to one full 2.497 GB generation per coordinator at 100k for a faithful
first implementation, plus mutation invalidation and remote fallback.

The packet explicitly labels the 1.445 GB compact figure as an unmeasured
lower-envelope estimate, not evidence. Task 198 must measure actual bytes and
may not stack compact packing into the faithful causal A/B.

Please review the quantified ceiling, rejection rationale, DML/lifecycle
boundary, and whether the selected family is narrow enough for one follow-up.
