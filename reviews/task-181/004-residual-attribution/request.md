---
task: 181
packet: 004-residual-attribution
role: coder
status: open
date: 2026-07-15
head: e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8
---

# Review request: residual-attribution non-trigger

Task 181 Phase 4 does not trigger. The best bounded candidate reaches 0.9625
distinct recall at 100k, versus 0.9970 for the reproduced owner oracle: a
0.0345 gap, well outside the required within-0.0050 trigger.

The Wilson intervals are disjoint in the same direction. No exact-neighbor
traversal arm, new quantizer, OPQ, or graph change was run. Phase 5 proceeds
with entry membership and RaBitQ traversal unchanged.
