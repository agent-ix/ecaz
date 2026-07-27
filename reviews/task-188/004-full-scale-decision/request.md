---
task: 188
packet: 004-full-scale-decision
role: coder
status: open
date: 2026-07-26
head: c1c43a9bf
---

# Review request: Task 188 full-scale decision

The required BW4-control versus BW8-candidate suite completed at 10k/50k/100k
with recall, warm latency, storage, build, head, topology, and engagement
artifacts. The exact head seed digest was held constant within each scale.

Decision: **select BW8 as the sole follow-up research candidate; do not change
production defaults or persisted formats in Task 188.**

BW8 is recall-neutral at 10k, improves recall by 0.0025 at 50k and 0.0065 at
100k, and has zero storage delta at every scale. The 50k mean/p95 regression
(+7.90/+21.80 ms) and the 100k p95 regression (+9.40 ms) are explicit
acceptance concerns for the follow-up task. No other candidate is advanced,
and no productionization is claimed here.

The packet-local summary and structured suite results are listed in
`artifacts/manifest.md`.
