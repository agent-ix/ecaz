# Task 187: ec_distann Traversal Transport Optimization

Status: **proposed, conditional on Task 184** (2026-07-19). Priority: P2
latency follow-up.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`. This task
owns `TRAV-01` through `TRAV-15` and `TRAV-20` through `TRAV-27`.

## Why

Task 183 measured traversal at 7.918 ms/query, 19.70% of 100k wall mean. That is
the second-largest current stage, but Task 184 must first reduce or reattribute
the larger 26.955 ms materialization stage. Traversal currently spans owner
graph reads, RaBitQ scoring, PostgreSQL row/array encoding, concurrent remote
transport, coordinator decode, frontier work, and hop synchronization; the
aggregate timer cannot select among them.

## Goal

Refresh the post-Task-184 end-to-end profile, decompose traversal, and select at
most one bounded traversal/transport optimization for isolated A/B. Preserve
recall, graph, ordering, epoch/failure semantics, and total work bounds.

## Entry gate

Task 184 must provide its final production-path profile and disposition. If
traversal is no longer a material share, close this task with a documented
conditional skip. Otherwise freeze the retained materialization path and index
generation before selecting a traversal candidate.

## Phase 1: attribution

At 100k split non-overlapping time and work into:

1. coordinator frontier and owner partitioning;
2. connection/session/prepared-state work;
3. request encoding and bytes;
4. owner directory/graph reads and decode;
5. owner approximate/exact scoring;
6. response encoding and bytes;
7. transport wait and per-owner straggler distribution;
8. coordinator receive/decode/frontier insertion; and
9. hop count, expansion batch widths, nodes requested/returned, cache hits,
   and repeated node/page reads.

Counters are feature-gated and reset after warmups.

## Phase 2: candidate selection

Pre-register one isolated candidate only after attribution. Eligible families:

- immutable graph/node caching (`TRAV-02`--`TRAV-04`);
- packed response/ID/code representation (`TRAV-05`--`TRAV-07`);
- bounded hop fusion/pipelining (`TRAV-08`--`TRAV-15`);
- graph-read/frontier/allocation locality (`TRAV-20`--`TRAV-24`); or
- runtime/query-state/straggler cleanup (`TRAV-25`--`TRAV-27`).

Replicated graph layers and gateway copies (`TRAV-28`--`TRAV-30`) belong to
Task 190. Adaptive recall/search-budget policies belong to Task 188.

## Isolated and full-scale evidence

Run baseline/candidate on a byte-identical fresh 100k generation. Record recall
and result identity, complete stage/counter movement, warm latency distribution,
storage/cache, topology, remote engagement, failure behavior, and release
provenance. Only an end-to-end useful candidate proceeds to 10k/50k/100k.

All matrices use checked-in `ecaz bench suite` configs. A stage-local win that
does not improve end-to-end mean/tails is rejected.

## Decision

Advance at most one candidate with material end-to-end benefit, preserved
recall/semantics, explicit work/cache/wire caps, and no unresolved protocol or
fallback choice. Production protocol/format changes require a separate task
and ADR review. Otherwise STOP.

## Required review packets

1. `reviews/task-187/001-post-materialization-baseline/`;
2. `reviews/task-187/002-traversal-attribution/`;
3. `reviews/task-187/003-isolated-candidate/`;
4. `reviews/task-187/004-full-scale-decision/`.

## Non-goals

- Payload materialization changes owned by Task 184.
- Entry-head policy/capacity owned by Tasks 185--186.
- Graph construction or adaptive search policy owned by Task 188.
- Codec replacement owned by Task 189.
- Replication, placement, or binary-RPC architecture owned by Task 190.

## References

- Task 183 packet 005 and Task 184 final evidence.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
- FR-079, FR-081, ADR-085, and NFR-017 through NFR-020.
