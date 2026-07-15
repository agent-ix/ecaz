---
task: 181
packet: 003-fixed-cap-policy-screen
role: coder
status: open
date: 2026-07-15
head: e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8
---

# Review request: fixed-cap landmark and hierarchy screen

Task 181 Phase 2 is complete. The fixed-cap screen in
`artifacts/manifest.md` compares all preregistered policy families on the real
100k corpus using exact scoring of 4,096 landmarks.

Disjoint-training landmarks are the only meaningful win: recall rises from
0.9275 to 0.9625, owner top-32 membership rises from 4.70% to 55.03%, and the
zero-representation fraction falls from 43.5% to 1.5%. Geometry coverage alone
is recall-flat at 0.9270; graph landmarks regress to 0.9160.

Because no fixed policy reached 0.9900, the required bounded-hierarchy suite
was triggered with explicit first-level, opened-region, second-level-score,
and returned-seed caps. It regressed to 0.9145 recall at 98.1 ms p50; even
exact scoring of all 16,384 geometry landmarks reached only 0.9440 at 46.7 ms.

The hierarchy is rejected. The frozen full-scale candidate is the 4,096-entry
disjoint-training landmark set with exact bounded scoring: at most 4,096
landmarks scored and 32 seeds returned, with BW4/H100 and RaBitQ traversal
unchanged.
