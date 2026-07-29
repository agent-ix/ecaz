---
id: NFR-022
title: Distann A/B Control Validity
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-022: Distann A/B Control Validity

## Statement

Every ec_distann candidate A/B SHALL use a control that itself satisfies
[NFR-021](./NFR-021-distann-distribution-invariant.md).

A candidate SHALL be screened for NFR-021 admissibility at **pre-registration**,
before any measurement is taken. An inadmissible candidate SHALL NOT be
benchmarked, promoted, or recorded as a program baseline, regardless of its
measured effect on latency, recall, or any other metric.

Single-instance access-method anchors (`ec_ivf`, `ec_hnsw`, `ec_diskann`) and
non-conforming lanes MAY be reported as context. They SHALL NOT be the control
against which a PROMOTE, STOP, or ITERATE decision is made, and they SHALL NOT
be recorded as the program's latency or recall baseline.

This requirement does not demote latency. Latency remains a primary ec_distann
objective; it is to be measured against the distributed architecture, not
against a design that abandons it.

## Scope

- Applies to: every ec_distann task that records a PROMOTE, STOP, ITERATE, or
  ACCEPT decision from measurement, and to every roadmap candidate ledger entry
  whose status is set by such a decision.
- Applies to the control arm, the candidate arm, and any arm whose result is
  carried into a task disposition.
- StR-008's single-instance economics goal is unaffected: the IVF/HNSW anchors
  remain the product comparison. This requirement governs which arm decides the
  *engineering* question, not what the product must ultimately beat.
- A task MAY compare a non-conforming lane deliberately — to quantify what the
  distribution invariant costs, for example — provided the packet states that
  purpose, labels the lane non-conforming, and does not derive a production
  default from it.

## Rationale

A control that does not satisfy NFR-021 is not the system under optimization. At
any corpus size that fits on one machine, a single-node index will win a latency
comparison by construction; measuring against it produces a result that is
arithmetically true and architecturally meaningless, and — because the
comparison is repeatable — it produces the same misleading result at every
scale the standard sweep covers.

The failure mode this prevents is documented. Task 199 promoted a
coordinator-resident full-graph traversal replica on a 15.9%/14.0%/17.0% warm
mean improvement over the owner-traversal path, and that replica arm was
subsequently recorded as "the current latency control" for the program, so
forward optimization work inherited a control in which search is not
distributed. The promotion cleared every gate then in force: recall parity,
latency Pareto, and an "explicitly accepted storage envelope". No gate asked
whether the winning arm was still the architecture.

Scale-model benchmarking makes this rule necessary rather than merely tidy. The
standard sweep runs 10k/50k/100k across a small roster precisely so the
distributed design can be exercised without the resources a target-scale corpus
would demand. That is sound methodology, and it depends entirely on candidates
preserving the properties the model represents. A candidate permitted to consume
a resource that does not exist at target scale invalidates the model rather than
being measured by it.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| candidate pre-registrations recording an explicit NFR-021 admissibility verdict | 100% | no exceptions | packet `request.md` audit |
| decision-bearing A/B controls satisfying NFR-021 | 100% | no exceptions | packet manifest audit |
| non-conforming lanes present in a run and labeled as such | 100% | no exceptions | suite arm labels in `results.jsonl` |
| roadmap ledger entries whose status derives from a non-conforming control | 0 | 0 | ledger audit |

## Verification

Each ec_distann review packet's `request.md` records the NFR-021 admissibility
verdict for its candidate before its measurement section. Each packet manifest
identifies its control arm and that arm's NFR-021 status. The suite labels every
arm's conformance in `results.jsonl` so an audit can be run mechanically rather
than by reading prose. A decision recorded against a non-conforming control is a
finding, and the disposition it produced is reopened.

## Dependencies

- **Upstream**: [StR-006](../stakeholder/StR-006-benchmark-evidence-discipline.md),
  [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
- **Related**: [NFR-021](./NFR-021-distann-distribution-invariant.md)
  (the admissibility criterion),
  [NFR-007](./NFR-007-benchmark-provenance.md) (evidence provenance),
  [NFR-017](./NFR-017-distann-latency-recall-gate.md) (which already excludes
  replicated-full-index lanes from satisfying the gate)
