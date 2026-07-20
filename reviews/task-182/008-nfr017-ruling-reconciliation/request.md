---
task: 182
packet: 008-nfr017-ruling-reconciliation
role: coder
status: open
date: 2026-07-19
head: 6d4870b6f
---

# Review request: reconcile NFR-017 with the stakeholder ruling

The second independent Task 182 closeout review accepted the opt-in trained
head promotion and identified a cross-cutting documentation inconsistency:
NFR-017's prose treated `0.999` recall and the `37.6 ms` IVF anchor as
aspirational, while its measurement table still called them thresholds.

Commit `6d4870b6f` resolves that finding without changing production behavior
or benchmark results:

- records the stakeholder's 2026-07-17 ruling that `0.999` is not an enforced
  acceptance gate and that the complete best-effort relative Pareto result is
  the decision basis;
- renames the numerical table columns to aspirational target and comparison
  reference;
- preserves the values as visible context rather than deleting them;
- separates those references from the mandatory FR-078 physical-topology
  validity prerequisite; and
- updates the matched-recall rule so a run below `0.999` still reports its
  Pareto frontier and closest-recall comparison without being automatically
  rejected.

No test or benchmark was run because this is a specification-only
reconciliation and changes no executable surface. Please review consistency
with the stakeholder ruling and the second-review findings in packet 007.

