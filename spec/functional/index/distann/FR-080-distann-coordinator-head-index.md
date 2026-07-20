---
id: FR-080
title: Distann Coordinator Head Index
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-080: Distann Coordinator Head Index

## Description

The coordinator SHALL maintain an in-memory head index — a Vamana graph over
a bounded sample of the global graph's entry region — so a query's first
hops execute locally with zero network round trips and hop rounds start deep
in the correct region of the global graph.

## Behavior

- At build time, the pipeline SHALL collect an entry-region sample of up to
  `head_index_cap` (C) vectors: a breadth-first traversal from each build
  shard's entry medoid over that shard's graph, bounded by hop radius, with
  the per-shard samples unioned (guaranteeing every shard's region is
  represented). Vamana graphs are single-layer; "entry region" means
  BFS-near the medoid, not a layered structure. The sample SHALL be
  persisted with the epoch as an epoch-versioned object in the index
  relation, listed in the epoch manifest alongside placement metadata
  ([FR-082](./FR-082-distann-epoch-lifecycle.md)). A single-shard
  (monolithic) build is the degenerate case: one medoid, one BFS sample.
- An explicit `training_landmarks_exact` generation MAY instead select the same
  bounded cap from exactly 200 ordered, finite, dimension-matched training
  queries supplied by a PostgreSQL relation. The builder SHALL rank each
  query's top 32 source RaBitQ codes, frequency/rank/vec_id order the union, and
  deterministically fill unused slots from geometry landmarks. The relation is
  build input under the build snapshot; its canonical query digest, count,
  policy, and selected head digest SHALL be fingerprint-bound. Evaluation-query
  training and server-local file inputs are forbidden.
- The coordinator SHALL construct the in-memory head index from the persisted
  sample on first use per epoch (reusing the in-memory Vamana builder used
  by the SPIRE top-graph, via **extract-to-shared** — the pure builder is
  lifted into a shared module, not forked and not edited in place under
  SPIRE's spec ownership) and cache it keyed on the exact `(index_oid,
  logical_index_uuid, build_id, epoch_fingerprint)` identity. A cold fill SHALL
  validate the immutable candidate/descriptor/head-sample digest chain before
  insertion. A hit may reuse only the validated descriptor digest and head
  graph; raw conninfo, relation handles, active-pointer state, and scan tokens
  SHALL NOT be cached. Each backend retains at most two immutable epoch entries
  and uses LRU eviction. The cache SHALL have a Userset off switch for A/B
  measurement and diagnosis; disabling it restores cold validation/construction
  on every scan without changing results.
- A query SHALL apply the active generation's bound head policy first; its best
  results seed the hop-round frontier of
  [FR-081](./FR-081-distann-query-orchestration.md). Legacy/current generations
  search the persisted Vamana head graph. `training_landmarks_exact`
  generations exact-score at most C persisted vectors and return the
  policy-bound prefix of at most 32 seeds. Unknown or inconsistent policy metadata fails
  closed.
- Head-index construction SHALL be deterministic under a fixed seed.
- If the persisted sample is missing or fails to decode, scans SHALL error
  (strict policy — a silent medoid-entry fallback would change recall
  without a signal).

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-080-CON-1 | Head-index memory SHALL be bounded by C × (vector bytes + graph overhead); C is a reloption with a documented default | Memory | Analysis + unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-080-AC-1 | Head-index search returns entry candidates without any remote call | Test |
| FR-080-AC-2 | Construction is deterministic for a fixed seed and epoch | Test |
| FR-080-AC-3 | Every build shard's region is reachable from the head sample | Test (property/BFS) |
| FR-080-AC-4 | Recall sensitivity to C is measured and recorded at M0 (informs the default) | Analysis (bench) |
| FR-080-AC-5 | Warm repeated scans reuse one validated epoch head graph, cache identity cannot alias OID/UUID/build/fingerprint changes, and bounded LRU eviction retains at most two entries per backend | Test + benchmark |
| FR-080-AC-6 | Trained policy input/count/digest and selected head are deterministic and fingerprint-bound; replay with different input fails | Test |
| FR-080-AC-7 | Existing version-1 options retain BFS/Vamana semantics while trained generations use bounded exact head scoring without benchmark features/GUCs | Test + benchmark |

## Dependencies

- **Upstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md);
  ADR-085 decision D3 (C policy)
- **Downstream**: [FR-081](./FR-081-distann-query-orchestration.md)

## Measured head-cap outcome

The Task 179 real three-owner PG18 suite in
`reviews/task-179/038-head-cap-sensitivity/` measured caps 64, 256, and 4096 at
10k, 50k, and 100k using 20 held-out queries (200 recall trials) and 20 latency
iterations per cell. Physical recall for 64 / 256 / 4096 was respectively
0.995 / 0.995 / 1.000 at 10k, 0.975 / 0.980 / 0.980 at 50k, and
0.920 / 0.945 / 0.950 at 100k. All nine cells had exact disjoint topology and
two proven remote owners. The 100k result rejects 64 and retains the D3 default
of 4096 over 256 for its final 0.005 recall increment; warm physical p50 at
4096 was also no worse in this matrix (70.7, 100.8, and 78.9 ms).
