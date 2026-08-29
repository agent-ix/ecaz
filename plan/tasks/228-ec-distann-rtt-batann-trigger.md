# Task 228: ec_distann RTT and BatANN Reopen Trigger

> **Tracking moved to GitHub (2026-08-29):** [agent-ix/ecaz#101](https://github.com/agent-ix/ecaz/issues/101)
> on [Project 19](https://github.com/orgs/agent-ix/projects/19), under EPIC #95.
> The Status header below is frozen; status updates land on the issue.
> Review packets remain under `reviews/task-228/`.

Status: **proposed, measurement-only** (updated 2026-08-23). Priority: P2
architecture gate after Tasks 222--237.

## Why

ADR-085 D4 defers BatANN baton/state passing until hop-round transport is at
least 50% of gate-relevant multinode p50. Current same-host evidence measures
roughly 4.1--5.0 ms/scan of traversal transport against approximately 20--28 ms
end-to-end, below the trigger. Tasks 222--233 can change owner service, payload
bytes, the end-to-end denominator, and the relevant BW/H operating point.
Tasks 234--237 replace the loopback/development transport gaps with bounded,
secure, observable production behavior. Measuring before those changes would
characterize a substrate that is not eligible for production.

The old `task-173-batann-specs` branch predates distribution restoration,
pushdown, lazy-10, current FR/NFR numbering, and the present conformance rules.
It is not an executable plan and cannot be merged as written.

## Goal

Measure the final pre-BatANN hop/RTT sensitivity on the optimized conforming
owner path and issue a reviewed GO/STOP decision on reauthoring—not
implementing—the BatANN program.

## Entry gate

1. Tasks 222--233 have reported, and the current materialization/search/storage
   control is frozen.
2. Tasks 234--237 are review-closed, so every RPC is bounded/cancellable, the
   real-network cell uses the production TLS/secret substrate, and EXPLAIN plus
   suite metrics expose the required protocol counters.
3. The measurement uses the selected current BW/H point and the same release,
   corpus, query, topology, and cache protocol across delay cells.
4. Any matrix/sweep is an `ecaz bench suite` configuration. If injected-delay
   support is missing, extend the suite runner as a separate reviewed commit;
   do not add a one-off shell sweeper.

## Scope

- Measure same-host zero-delay and controlled per-hop delay cells, plus a real
  multi-host production-TLS cell.
- Capture head, traversal, materialization, and maintenance message counts and
  actual encoded request/response bytes, including PostgreSQL framing where
  measurable.
- Run concurrency 1/2/4/8/16 and report backpressure, live/pooled connection
  counts, opens/reuse/evictions, queueing, and owner saturation.
- Report end-to-end mean/p50/p95/p99, traversal rounds, transport wait,
  straggler spread, owner service, materialization, throughput, recall, and
  injected/observed delay provenance.
- Compute the transport share at every gate-relevant operating point.
- Record GO only if the ADR-085 D4 >=50% trigger is met and narrower conforming
  work is exhausted; otherwise retain the deferral.
- If GO, create a new spec-authoring task with non-conflicting ADR/FR/NFR/task
  identifiers and current NFR-021/NFR-022 controls.

## Non-goals

- Implementing relay state, baton passing, mailboxes, stack/direct return, or
  locality-aware placement.
- Merging or rebasing the stale Task 173 branch as a substitute for reauthoring.
- Claiming real-network behavior from an unlabeled same-host loopback run.

## Acceptance

1. Every cited row is suite-produced and traces to packet-local structured
   results with release and delay provenance.
2. Recall and bounded-work semantics remain unchanged across delay cells.
3. The packet records the D4 numerator, denominator, share, and explicit
   GO/STOP disposition.
4. GO creates planning/spec work only; BatANN implementation remains separately
   authorized and benchmark-gated.

## Required review packets

1. `reviews/task-228/001-plan/`
2. `reviews/task-228/002-suite-delay-surface/` (only if runner support changes)
3. `reviews/task-228/003-rtt-matrix-and-decision/`

## References

- ADR-085 D4
- NFR-017 injected-latency obligation
- Tasks 194, 216, and 222--237
- Historical branch `origin/task-173-batann-specs` (superseded planning only)


