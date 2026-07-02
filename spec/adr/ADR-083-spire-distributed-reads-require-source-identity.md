---
type: ADR
id: ADR-083
title: "Distributed SPIRE Reads Require the Source-Identity Provider"
status: ACCEPTED
impact: Governs Task 137 distributed result deduplication. Affects ADR-055, ADR-063, the local multi-instance fixture, distributed load tooling in ecaz-cli, and every multi-instance SPIRE benchmark lane.
date: 2026-07-02
---
# ADR-083: Distributed SPIRE Reads Require the Source-Identity Provider

## Context

Task 131 packet 027 identity artifacts proved that the distributed SPIRE
production read surface returns the same corpus row multiple times inside a
single top-k result (183/200 duplicate-containing top-10 results at 10k,
1000/1000 at 50k, worst case 4 distinct ids for k=10). Task 137 owns the fix.

The confirmed mechanism is not a merge bug. The final top-k merge dedupe key
(`remote_search_candidate_dedupe_key`) is correct for both identity scopes
ADR-055/ADR-063 define:

- global `0x02` vec_ids dedupe across nodes;
- node-local `0x01` vec_ids dedupe only within their origin node, because
  local sequences from independently built per-node indexes can collide and
  merging them by bare vec_id would collapse *different* rows.

The defect is deployment-shaped: the local multi-instance fixture (and the
matching AWS lane) builds independent per-node remote indexes from overlapping
row slices — boundary-replica leaf assignments place the same corpus row on up
to all three remote nodes — while every index runs with node-local vec_ids.
With no shared identity, the merge provably cannot collapse replicas of the
same row, and ADR-063 already states such indexes "cannot make cross-node
replica dedupe claims". The read path even ships the matching diagnostic
(`requires_global_vec_id` in the boundary-replica identity snapshot).

## Decision

1. **Distributed SPIRE read surfaces (any topology where more than one node
   can serve the same corpus row) MUST run with the ADR-063 source-identity
   provider engaged** (`source_identity = 'include'` plus an INCLUDE identity
   column), so every copy of a logical row carries the same global `0x02`
   vec_id and the existing merge dedupes it.
2. The corpus loader owns provider wiring for benchmark and fixture lanes:
   `ecaz corpus load --reloption source_identity=include` creates the corpus
   table with a stored generated identity column
   (`sha256(int8send(id))[..16]`, a 16-byte bytea payload) and builds the
   index with `INCLUDE (source_identity)`. The derivation matches the loader's
   static shard-routing identity, and the stored-generated-column shape is the
   expression-derived identity form ADR-063 blesses for v1.
3. The multi-instance fixture and its suite configs pass the reloption through
   to the coordinator and all remote loads. Multi-instance benchmark packets
   filed after this ADR MUST run with the provider on unless the packet is
   explicitly measuring the legacy local-identity behavior, and must state
   that exception in the packet manifest.
4. Node-local vec_id merge semantics stay exactly as ADR-063 specifies (scoped
   by node, no cross-node dedupe claims). No heuristic cross-node local-id
   dedupe is added: without a shared identity there is no sound key, and
   collapsing colliding local sequences would corrupt results.

## Consequences

- Multi-instance recall/latency baselines recorded before this ADR measured a
  surface that returned non-distinct top-k results; Task 138 owns re-scoring
  that history with `distinct_recall@k`.
- `k` distinct results become a real guarantee of the distributed surface only
  when the provider is on; the boundary-replica identity snapshot diagnostic
  remains the operator check.
- Storage grows by the 16-byte identity per indexed row plus the INCLUDE
  payload; the Task 137 packet accounts for the measured delta.
- Delta-insert DML on distributed indexes keeps allocating node-local ids only
  where the provider is off; provider-on indexes reject rows that cannot
  produce a valid identity, per ADR-063 fail-closed rules.
