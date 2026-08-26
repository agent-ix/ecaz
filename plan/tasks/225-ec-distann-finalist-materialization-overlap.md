# Task 225: ec_distann Finalist Materialization Overlap

Status: **proposed, conditional on its own measured finalist-stability and
hideable-RTT premise; Task 224's no-finalist STOP neither satisfies nor rejects
this entry gate** (updated 2026-08-25). Priority: P2 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
MAT-07, MAT-08, MAT-35, and MAT-36.

## Why

Lazy-10 correctly delays payload work until the executor consumes proven
ranked rows, but traversal and final payload materialization remain sequential.
If Tasks 222--224 leave materialization wall time dominated by one remote
round-trip rather than owner CPU/heap work, overlap or piggybacking may hide
that residual. It also risks fetching unstable finalists or coupling traversal
failure to executor semantics, so it requires a stability gate before code.

## Goal

Measure finalist stability and the avoidable materialization round-trip, then
advance at most one bounded overlap family: final-round prefetch, next-window
pipeline, combined final-rank/materialize endpoint, or final-expansion
piggyback.

## Entry gate

1. Tasks 222--224 have reported and define the current control.
2. Per-query diagnostics compare the final proven top-k with candidates after
   the penultimate and final traversal rounds and report useful-prefetch rate,
   wasted rows/bytes, owner distribution, and cancellation exposure.
3. A candidate proceeds only if expected hidden wall time is at least 1 ms or
   5% of warm mean and wasted work remains under a pre-registered fixed bound.

## Scope

- Instrument stability without changing selection or payload semantics.
- Select exactly one candidate family before implementation.
- Keep all work bounded by the existing proven-prefix/deepening ceiling.
- Preserve fail-closed owner errors; no partial-result success or silent
  degradation.
- Screen at same-generation 100k, then run 10k/50k/100k only for a useful win.

## Non-goals

- Speculative cross-query caching or unbounded prefetch.
- Changing search budget and overlap in one A/B.
- BatANN relay/state passing.
- Weakening lazy-10's qual-driven correctness contract.

## Acceptance

1. Stability evidence justifies one candidate or closes the family without
   implementation.
2. Wasted rows/bytes, cancellation, restart, outage, and qual-deepening behavior
   are explicit and bounded.
3. Any winner moves end-to-end mean/tails and passes the full semantic matrix
   plus 10k/50k/100k recall/latency/storage evidence.

## Required review packets

1. `reviews/task-225/001-plan/`
2. `reviews/task-225/002-finalist-stability/`
3. `reviews/task-225/003-isolated-overlap-candidate/`
4. `reviews/task-225/004-full-scale-decision/` (only after a useful screen)

## References

- Task 191 lazy-10 productionization
- Tasks 222--224
- Roadmap MAT-07 / MAT-08 / MAT-35 / MAT-36

