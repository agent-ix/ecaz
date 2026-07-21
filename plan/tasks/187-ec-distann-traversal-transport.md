# Task 187: ec_distann Traversal Transport Optimization

Status: **complete — STOP, no candidate** (2026-07-21). Priority: P2 latency
follow-up. Task 191 promoted and release-validated lazy10 as the retained
production baseline.

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

Task 184 provided its profile and PROMOTE disposition. Task 191 landed and
validated the selected materialization path. Traversal remains material at
100k: 7.849 ms against 23.70 ms warm wall mean (33.1%), so the conditional
skip does not apply and Phase 1 attribution is executable.

### Retained Task 191 baseline

- production `training_landmarks_exact`, cap 4,096, exact landmark scoring,
  32 returned seeds, BW4/H100, graph degree 32, RaBitQ neighbor scoring;
- deterministic global-ranked payload windows fixed at 10, with no production
  tuning GUC;
- 100k physical recall 0.9625 (95% Wilson 0.9532–0.9700), warm latency
  mean/p50/p95/p99/max 23.70/23.50/27.20/28.00/28.10 ms;
- 100k physical generation 2,496,626,688 bytes, head digest
  `50261d7627471fa3329535cd017ead6102cb220c62ca12dc9715178d05333b54`,
  seed digest
  `488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`;
- staged query SHA-256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`;
  and
- retained evidence:
  `reviews/task-191/003-production-full-scale/artifacts/full-run/production-ab-100k/distann-multinode-summary.log`.

Phase 1 refreshed attribution on a fresh byte-identical generation. The 100k
physical arm measured 7.468 ms traversal in a 22.40 ms warm mean; remote owner
expansion was 6.174 ms, local expansion 1.230 ms, and derived coordinator /
frontier remainder 0.065 ms. No bounded candidate was selected because the
dominant remote transport path is not decomposed finely enough to attribute a
cache, packing, hop, locality, or straggler change safely.

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
and ADR review. Decision: STOP. No production code changed; the next task
should add per-owner transport encode/wait/decode/straggler counters before
attempting one isolated transport optimization.

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
