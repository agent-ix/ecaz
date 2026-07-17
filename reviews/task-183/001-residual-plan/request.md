---
task: 183
packet: 001-residual-plan
role: coder
status: open
date: 2026-07-17
head: 973f4dc3db57650c3a6f8d41818880f146e87896
---

# Review request: residual recall and latency plan

Task 183 is the measurement-first follow-up after Task 182's production-path
landmark A/B. Task 182 is now complete, and this packet freezes its immutable
production baseline before experiments begin.

The plan separates four uncertainties:

1. byte-identical-seed RaBitQ versus exact-neighbor traversal attribution;
2. better trained landmark coverage at the same cap 4,096;
3. conditional trained cap 8,192 and one explicitly bounded query-conditioned
   routing design; and
4. profile-driven latency changes measured one at a time.

Every behavioral candidate receives isolated attribution before the final
10k/50k/100k suite. A useful 100k winner adds 1m scaling confirmation when the
staged fixture is available and valid. The owner scan remains diagnostic and
O(N), and all production changes/defaults remain out of scope.

The decision uses the complete relative recall/latency/storage/build Pareto
result against Task 182. Proposed NFR-017 values are reported only as context.

The inherited baseline is production `training_landmarks_exact`: cap 4,096,
exact scoring of all persisted landmarks, at most 32 returned seeds, BW4/H100,
graph degree 32, RaBitQ neighbor traversal, and exact final rerank. It uses the
same staged corpora, rows 201--400 as disjoint training queries, and rows
1--200 as held-out evaluation queries. Task 182 measured distinct recall
0.9990 / 0.9685 / 0.9625 and warm p50 38.5 / 39.3 / 41.4 ms at
10k/50k/100k. Its owner oracle measured 0.9995 / 0.9970 / 0.9970 recall but
remains O(N) and non-selectable.

Please review the phase ordering, same-seed contract, training/evaluation
separation, bounded routing caps, per-change A/B attribution, and handoff
boundary. The durable plan is `plan/tasks/183-ec-distann-residual-recall-latency.md`;
`artifacts/manifest.md` records the planning provenance.
