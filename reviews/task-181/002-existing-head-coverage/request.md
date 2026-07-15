---
task: 181
packet: 002-existing-head-coverage
role: coder
status: open
date: 2026-07-15
head: e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8
---

# Review request: existing-head coverage audit

Task 181 Phase 1 is complete. Please review the benchmark-only diagnostic
surface and the 100k loss decomposition in `artifacts/manifest.md`.

The unchanged cap-4096 head reproduces Task 180: persisted and exact membership
both reach 0.9275 distinct recall, versus 0.9970 for the O(N) owner oracle. The
new owner/head oracle shows only 4.703% of owner top-32 seeds present in the
head, with 43.5% of evaluation queries having zero owner top-32 representation.
The exact-versus-persisted equality therefore attributes the primary loss to
landmark membership rather than approximate head search.

The implementation is feature-gated and does not change production defaults,
persisted format, production head construction, or normal seed selection. The
next registered action is the fixed-cap geometry, graph, and disjoint-training
policy screen in packet 003.
