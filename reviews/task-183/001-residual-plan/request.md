---
task: 183
packet: 001-residual-plan
role: coder
status: open
date: 2026-07-16
head: 07a16b86e235a380d539d55be0a26fbfbc2e6e8c
---

# Review request: residual recall and latency plan

Task 183 is the measurement-first follow-up after Task 182's production-path
landmark A/B. It does not start experiments until Task 182 freezes the actual
production baseline.

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

Please review the phase ordering, same-seed contract, training/evaluation
separation, bounded routing caps, per-change A/B attribution, and handoff
boundary. The durable plan is `plan/tasks/183-ec-distann-residual-recall-latency.md`;
`artifacts/manifest.md` records the planning provenance.
