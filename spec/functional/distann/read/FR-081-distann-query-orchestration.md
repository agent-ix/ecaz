---
id: FR-081
title: Distann Query Orchestration and Scan Semantics
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-081: Distann Query Orchestration and Scan Semantics

## Description

A top-k scan SHALL execute as a coordinator-driven beam search: a head
seeding phase — on a multi-owner roster a sharded head fan-out and
deterministic seed merge per
[FR-080](./FR-080-distann-coordinator-head-index.md), degenerating to a
local head descent on a single owner — followed by at most H batched hop
rounds, where each round expands
the best BW unvisited frontier candidates via one parallel
`ec_distann_expand_nodes` call per owning node, deriving an L-bounded
threshold and per-response `l = L` window, and merging code-scored
neighbors into the beam and exact distances into the result heap.

## Behavior

- Graph traversal and exact-distance ranking SHALL be eager (ADR-056 pattern):
  the orchestration loop runs at rescan and produces a bounded ranked candidate
  set. Final payloads SHALL NOT be eagerly fetched for that entire set.
  `CustomScan` cursor demand SHALL materialize the fixed global-ranked windows
  specified by FR-079, evaluate coordinator quals, and deepen the eager search
  bar only when the current proven prefix cannot satisfy the executor.
- Per hop round the coordinator SHALL: select the best BW unvisited beam
  candidates; group them by owning node
  ([FR-078](../build/FR-078-distann-hash-placement.md)); issue the per-node
  expansion calls in parallel over the pooled transport (operational/security
  posture per [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md),
  the lifted SPIRE transport contract); derive `t = peek_worst(H_C)` from the
  current L-bounded live heap and `l = L` for each owner response, then
  pass them to each owner for Algorithm 1 prune/sort/truncate; merge returned
  neighbor candidates (code distances) into the beam and returned exact
  distances into the top-k heap; and mark expanded nodes visited.
- When gateway copies are populated
  ([FR-086](./FR-086-distann-gateway-copies.md)), the coordinator SHALL name
  the gateway-cached ids in the expansion request's `skip_neighbor_vec_ids`.
- When an owner omits neighbor payloads for skip-listed ids, the coordinator
  SHALL reconstruct those rows' candidate halves from its gateway copies.
- The coordinator SHALL re-apply the batch candidate limit `l = L` once
  across the merged batch (owner-supplied plus reconstructed candidates
  together), preserving owner-only result equivalence. Exact distances and
  tombstone authority still come from the owner rows.
- Head seeding on a multi-owner roster SHALL follow the persisted head shape:
  a membership-only head fans a per-shard head-search request to every
  head-shard holder and merges at most `seed_count` seeds per holder
  deterministically, per [FR-080](./FR-080-distann-coordinator-head-index.md);
  the merged seeds feed the hop-round frontier. The coordinator-local descent
  is only the single-owner degenerate shape.
- `L` is an explicit retained-candidate heap parameter with
  `L >= max(BW, k)`. After each merge the
  coordinator SHALL retain only the best `L` live, unexpanded candidates and
  SHALL derive `t` from the current L-th candidate rather than from the total
  future `BW × H` budget. The value of `L` and the per-response `l = L` SHALL
  be included in benchmark provenance; the cost is O(L) coordinator frontier
  state and at most L returned neighbors per owner response.
- The loop SHALL terminate after H rounds, or earlier when the beam's best
  unvisited code distance cannot improve the current kth exact distance
  (convergence early-exit).
- The scan SHALL treat BW × H as the hard expansion cap
  ([NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)).
- Visited-set dedupe SHALL use vec_id as its key.
- The scan SHALL NOT expand one vec_id twice in one attempt.
- Final results SHALL be ordered by exact distance; no separate rerank
  round-trip is performed (exact distances arrive with expansion responses).
- The scan SHALL draw results only from expanded records.
  Head-index candidates enter results only through their own expansion.
- If the beam exhausts (no unvisited candidates remain) before k results
  accumulate, the scan SHALL return the fewer-than-k results as a complete
  result (scan exhaustion is not a fault); empty index → zero rows.
- The single-node local-expansion form of this loop is the first
  implementation slice (milestone M0), delivered before the remote transport
  exists; the remote form retargets the same loop.
- The scan SHALL surface per-query counters (rounds executed, records
  expanded, candidates code-scored, per-node batch sizes, pool reuse) via
  EXPLAIN and the bench pipeline step.
  *Implementation gap (Task 214 audit, F8): no EXPLAIN counter surface
  exists (`ExplainCustomScan` is unimplemented). Counters are emitted only as
  a NOTICE behind the `scan_profile_notice` debug GUC, and per-node batch
  sizes plus pool-reuse counters exist only in the benchmark feature build.
  The requirement stands; cross-reference
  [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  for the touch-bound counters this surface must carry.*
- Every remote connection SHALL have a nonzero connect deadline. Every remote
  call — lifecycle, expansion, materialization, head-shard search,
  gateway-routing export, and head-shard replica export/import — SHALL have
  both a nonzero client-side deadline and a remote `statement_timeout`; the
  coordinator SHALL check PostgreSQL interrupts before and after each awaited
  RPC. Timeout and cancellation SHALL fail the attempt without returning
  partial results. This includes a materialization failure after one or more
  earlier cursor windows.
  *Implementation gap (Task 214 audit, F9): expansion, materialization, and
  lifecycle calls conform via the deadline/interrupt wrapper. Four RPCs
  bypass it with bare awaits — head-shard search, gateway-routing export,
  head-shard export, and head-shard import — carrying no client-side timeout,
  no interrupt check, and no cancel token, so a stalled owner during head
  search can block the backend beyond its budget. The requirement stands for
  all six call classes.*
- The production materialization window SHALL be exactly 10 and SHALL have no
  production GUC or reloption. An eager override MAY exist only in a
  benchmark/test feature build. Installing the prior extension binary restores
  eager materialization without rebuilding an index because this policy changes
  no persisted index, row-tier, wire, or generation bytes.
- While the deployment is single-node, the same loop SHALL run with a local
  expansion function of identical signature (no transport).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-081-AC-1 | 2-node top-k results are identical to a single-node index built from the same corpus and seed | Test |
| FR-081-AC-2 | Records expanded per query ≤ BW×H in every benchmarked cell | Test (counter assertion) |
| FR-081-AC-3 | No vec_id is expanded twice within one scan | Test |
| FR-081-AC-4 | Early-exit never returns results different from running all H rounds | Test (A/B on fixed corpus) |
| FR-081-AC-5 | EXPLAIN reports the per-query traversal counters | Inspection |
| FR-081-AC-6 | A stalled connect or remote statement terminates within its configured nonzero budget, and cancellation is observed between RPC awaits without returning partial rows | Test |
| FR-081-AC-7 | Production builds use the fixed lazy-10 driver and expose no materialization-batch GUC; a feature build can select eager solely for matched A/B evidence | Inspection + Test (TC-040, TC-049) |
| FR-081-AC-8 | Unqualified and qual-driven payload reads obey NFR-019's fixed-window bounds, including across search deepening | Test (counter assertion) |

## Dependencies

- **Upstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-080](./FR-080-distann-coordinator-head-index.md) (sharded head seeding,
  as amended by ADR-087),
  [FR-086](./FR-086-distann-gateway-copies.md) (gateway-copy merge
  reconstruction); ADR-085 decision D9 (termination rule)
- **Downstream**: [FR-083](../lifecycle/FR-083-distann-dml-path.md); the bench gate
  ([NFR-017](../../../non-functional/NFR-017-distann-latency-recall-gate.md))
