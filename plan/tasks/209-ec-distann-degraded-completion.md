# Task 209: ec_distann Bounded Degraded Completion (Paper §4.2)

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **proposed** (2026-07-29). Priority: P2 tail latency.

Entry gate: after Tasks 205/206 report, since the straggler distribution they
change is exactly what this task acts on.

## Why

`DISTRIBUTEDANN` §2.4 uses hedged requests, tracks replica health across
requests, and allows "partial failures of batches of node reads in order to
reduce the tail latency normally associated with a high-fanout system". §4.2
measures the consequence and finds it safe:

> "The service experiences a graceful degradation in recall roughly proportional
> to the failure rate. This not only gives confidence in the reliability of the
> system, but also the performance stability, as we can safely timeout node
> scoring requests experiencing tail latencies without a significant adverse
> effect on recall."

Their Table 2: recall@5 90.8 / 89.7 / 88.8 / 87.5 / 87.0 at 100 / 99 / 98 / 97 /
96 percent node-scoring availability.

ecaz has no such mechanism. It fails closed, and `NFR-020` is explicit about why
(`:72-77`): a hop-round architecture's partial-result hazard would "silently
degrade recall — the exact class of silent wrongness ... that cost this project
weeks on the predecessor surface."

That reasoning is correct and is not being overturned. But `NFR-020-AC-6` already
anticipates this task:

> "Degraded completion, if introduced later, requires a follow-up FR with
> explicit opt-in and result labeling; the default path never degrades silently."

Task 194 measured straggler spread as a real cost (0.411 -> 0.736 ms/scan when
widening), and the paper's answer to stragglers is the one mechanism ecaz lacks.

## Goal

Implement §4.2's tail-latency behavior as an **opt-in, labeled** mode under
`NFR-020-AC-6`, with the default path unchanged and never silently degrading.

## Phases

1. **FR slice.** New FR for degraded completion: opt-in surface, result labeling
   that reaches the client (not only a counter), the exact contract for what
   "complete" means, and how a labeled partial interacts with the FR-081
   early-exit equivalence guarantee.
2. **Straggler timeout.** Per-owner deadline on an expansion round, with the
   round completing on the responses received and the result labeled degraded.
   The default remains fail-closed.
3. **Hedged requests and replica health.** §2.4's other two mechanisms, measured
   separately from the timeout.
4. **Degradation curve.** Reproduce §4.2's experiment: injected owner failure /
   slowness rates against recall, to establish that ecaz degrades proportionally
   rather than catastrophically. This is the evidence that makes the mode safe to
   offer at all.

## Benchmark gate

10k/50k/100k latency and tails with the mode off (must be byte-identical to
today) and on, plus the injected-failure degradation curve at 100k. Owner arm as
control. Recall must be reported at every injected rate, not only at zero.

## Required review packets

1. `reviews/task-209/001-contract/` — the FR, the labeling contract, and the
   NFR-020 reconciliation.
2. `reviews/task-209/002-implementation/` — code plus the fault drills.
3. `reviews/task-209/003-degradation-curve/` — the §4.2 reproduction.
4. `reviews/task-209/004-full-scale/` — off/on matrix and disposition.

## Non-goals

- Changing the default path to degrade. The default stays correct-or-error.
- Weakening `NFR-020`'s silent-wrongness prohibition. Labeling is the whole
  point: a degraded result that is not visibly degraded is the failure mode
  NFR-020 exists to prevent.
- Traversal budget or head changes.

## References

- `DISTRIBUTEDANN` §2.4, §4.2 and Table 2.
- `NFR-020` (`:72-77`, `AC-6`), `FR-081`, `NFR-019`.
- Task 194's straggler-spread measurement.
