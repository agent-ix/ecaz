# Task 184: ec_distann Remote Payload Materialization

Status: **proposed** (2026-07-17). Priority: P1 measured latency follow-up.

## Why

Task 183 profiled the retained Task 182 production policy on a fresh 100k
three-owner physical generation. Warm mean latency was 40.20 ms. Remote payload
materialization consumed 26.955 ms/query (67.05% of wall mean), while bounded
traversal consumed 7.918 ms, exact head scoring 2.272 ms, and seed selection
0.101 ms. Optimizing the entry head is therefore not the leading latency lever.

The current materialization stage begins after global candidate ranking and
ends when remote rows have been fetched and installed for output. Its aggregate
timer does not distinguish owner SQL execution, connection/request scheduling,
wire bytes and transport, coordinator receive/decode, datum copying, or work
spent materializing candidates that the executor never consumes. Those costs
must be separated before choosing a change.

## Goal

Attribute remote payload-materialization cost and select at most one bounded,
correctness-preserving optimization for production implementation. Demonstrate
the selected change in isolation against the current physical production path.

This task may add benchmark-only counters and a gated candidate implementation.
It must not weaken projection or qual correctness, failure semantics, snapshot
and generation fencing, global ordering, distinct-result identity, or bounded
work. No production default changes before the full confirmation decision.

## Phase 1: attribution

On the same retained 100k policy and physical topology as Task 183, split the
materialization path into non-overlapping stages:

1. candidate partitioning and per-owner request preparation;
2. connection acquisition and request dispatch;
3. owner-side statement execution and row lookup;
4. remote row encoding plus bytes returned;
5. coordinator receive/decode and datum ownership/copying;
6. result-map insertion and final payload association; and
7. candidates requested, rows returned, bytes returned, and candidates actually
   consumed by the executor.

Counters must be feature-gated, reset after warmups, and reported per timed
query. Explicitly identify nested timers. If transport and owner execution
cannot initially be separated, add a request-side timer and a server-side timer
whose boundary makes the remaining network/client residual derivable.

## Phase 2: candidate selection

Pre-register isolated candidates only after Phase 1 identifies their target.
Candidate families may include:

- reducing round trips or repeated statement/connection setup without changing
  requested rows;
- avoiding redundant encoding, decoding, or datum copies while preserving
  PostgreSQL memory-context ownership;
- reducing payload width only from planner-proven columns required by the
  target list, quals, identity, ordering, and failure checks, with fail-closed
  fallback when the projection cannot be proven; or
- bounded incremental payload batches when evidence shows many eagerly fetched
  candidates are unused, with deterministic deepening sufficient for filtered
  queries and an explicit maximum work bound.

Do not combine families in one A/B. Do not infer that `LIMIT k` permits fetching
only `k` rows when quals can reject candidates. Do not remove columns based on
SQL examples or benchmark projection alone. Any lazy or narrow-payload design
requires adversarial qual/projection tests, including filters that reject the
first candidate batch, toasted/varlena columns, nulls, system identity needs,
and mixed local/remote winners.

Select no candidate if attribution shows no bounded change with a credible
end-to-end benefit. A benchmark-only negative result is a valid STOP outcome.

## Phase 3: isolated A/B

For the selected candidate, run matched baseline/candidate cells on a
byte-identical index, corpus, query set, generation, and head policy. Record:

- distinct recall and Wilson interval;
- warm mean/p50/p95/p99/max latency;
- all Phase 1 stage times and work/byte counters;
- rows requested, returned, filtered, and consumed;
- build/publish time and physical/control/source/single-index storage;
- topology, remote engagement, query separation, and unanimous installed
  release provenance; and
- output identity for unfiltered queries plus semantic equivalence for the
  adversarial projection/qual matrix.

Use relative Pareto evidence, not an invented absolute latency gate. Reject a
candidate that trades away recall, result semantics, failure correctness, or a
meaningful amount of tail latency for a mean-only improvement.

## Full-scale confirmation

Only a useful isolated candidate proceeds to a checked-in `ecaz bench suite`
A/B at 10k/50k/100k. Use at least 200 held-out queries / 2,000 distinct top-10
trials and 50 warm latency samples after 10 warmups at concurrency 1. Measure
recall, latency, storage, construction, stage attribution, work/bytes, topology,
remote engagement, and release provenance at every scale.

Run 1m only after the candidate demonstrates a useful relative improvement at
100k and the staged corpus meets the same provenance requirements. If no
candidate is selected, record the conditional skip rather than running a
candidate-free scale matrix.

All matrices and multi-step runs use `ecaz bench suite`; extend the runner in a
separate checkpoint if it lacks a needed counter or arm. Evidence follows
NFR-007 and contains only compact cited artifacts.

## Decision

Advance to production only if one bounded candidate:

1. materially reduces end-to-end physical latency at deficient scales and the
   attributed target moves consistently;
2. preserves recall, ordering, projection/qual semantics, identity, snapshot
   and generation fencing, and failure behavior;
3. reports complete storage, bytes, work, construction, mean, and tail costs;
4. passes topology, remote engagement, query separation, and release
   provenance checks; and
5. leaves no unresolved batching, projection, fallback, format, or work-cap
   choice.

The candidate remains opt-in until this decision is recorded. A stage-local win
without an end-to-end win is not a production result.

## Required review packets

1. `reviews/task-184/001-materialization-plan/`: frozen Task 183 baseline,
   stage boundaries, counters, fixture, and candidate-selection contract;
2. `reviews/task-184/002-materialization-attribution/`: completed 100k profile
   and pre-registered isolated candidate or STOP;
3. `reviews/task-184/003-isolated-candidate/`: conditional implementation,
   correctness evidence, and same-generation A/B; and
4. `reviews/task-184/004-full-scale-decision/`: conditional 10k/50k/100k and
   1m evidence plus promote/iterate/stop decision.

## Non-goals

- Entry-head coverage, scoring, or seed-selection work from Task 183.
- Traversal codec or graph replacement.
- Owner-wide scans or unbounded incremental fetching.
- Projection pruning without planner proof and fail-closed fallback.
- Masking remote failures, weakening snapshot semantics, or returning partial
  results as complete.
- Task 167 incremental DML or Task 172 capacity/RTT characterization.

## References

- Task 183 packet 005: immutable stage profile.
- Task 183 packet 006: no-candidate decision and Task 184 handoff.
- Task 182: retained production head policy and full-scale baseline.
- ADR-085 and FR-078/FR-079: physical placement and row materialization.
- NFR-007 and NFR-017 through NFR-020.
