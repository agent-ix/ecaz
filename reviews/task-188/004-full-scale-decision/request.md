---
task: 188
packet: 004-full-scale-decision
role: coder
status: open
date: 2026-07-26
head: c1c43a9bf
---

# Review request: Task 188 full-scale decision

The historical packet-003 matrix used omitted variant batch fields and is
retained as eager-0 unbatched evidence only. The corrected final matrix and
paired recall evidence are in
[`005-batch10-reconfirmation`](../005-batch10-reconfirmation/request.md).

The required BW4-control versus BW8-candidate suite completed at 10k/50k/100k
with recall, warm latency, storage, build, head, topology, and engagement
artifacts. The exact head seed digest was held constant within each scale.

Decision: **accept BW8 as the sole isolated search-budget research candidate;
do not change production defaults or persisted formats in Task 188.**

The corrected batch-10 run is recall-neutral at 10k, improves recall by 0.0025
at 50k and 0.0065 at 100k, and has zero storage delta at every scale. Paired
per-query outcomes are 0/0/200 ties at 10k, 5 candidate wins versus 0 control
wins at 50k, and 7 versus 0 at 100k; the latter two bootstrap intervals are
positive. BW8 is also faster than BW4 in warm mean and p95 at all three scales.
No other candidate is advanced, and no productionization is claimed here.

The packet-local summary and structured suite results are listed in the
corrected packet's `artifacts/manifest.md`.

The remaining bounded-head gap to the owner oracle is not proven irreducible by
this search-budget screen. The unrun frontier/reachability/graph-quality
families remain unselected rather than silently deferred; ordering/neighbor
estimation residuals belong to Task 189, while architecture/transport residuals
belong to Task 190. Task 188 makes no claim that those families were refuted.
