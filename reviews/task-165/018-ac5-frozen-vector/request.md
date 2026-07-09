# Review request — Task 165: AC-5 frozen vec_id→vector under delete+VACUUM+reuse

**Branch:** `task-165-ec-distann-m3`. Closes FR-082-AC-5 with a real
delete+VACUUM+TID-reuse drill, demonstrating the guarantee holds **without** a
separate frozen-vector tier (which would reverse ADR-085 D11).

## Design rationale (best design, not D11 reversal)

AC-5 requires a vec_id's rerank vector to stay byte-identical, with no
base-table delete+VACUUM+TID-reuse causing a mis-rerank. Under ADR-085 D10
(nothing physically reclaimed within a Published epoch) + the AM's ambulkdelete/
VACUUM coordination, this already holds: a deleted record is tombstoned by
ambulkdelete (so it is never reranked), and a **live** record's heap TID is
never reclaimed (so its co-placed vector is frozen). Re-embedding vectors in the
index (a frozen tier) would double the vector storage D11 deliberately moved to
the heap — redundant under D10. So the drill *demonstrates* the property rather
than building the redundant tier.

## The drill (`ac5_frozen_vector_after_vacuum_reuse`)

1. Probe row 1's multi-node top-1 (id:distance), byte-exact.
2. On **every node**: `DELETE` a mid range, then `VACUUM dm` (ambulkdelete
   tombstones + reclaims the deleted rows' heap TIDs).
3. On every node: reinsert rows over the freed id range (may reuse the TIDs).
4. Re-probe row 1 (never touched) → must be byte-identical, no `EC_VECTOR_MISSING`.

## Evidence (`artifacts/distann-multinode-summary.log`, real 3× PG18)

```
ac5_frozen_vector_after_vacuum_reuse pass=true
live_retention_gate pass=true
concurrency_scan_insert_epochswap pass=true
RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0
recovery ... recovered=true
```

## Status: all 6 FR-082 sub-ACs demonstrated

AC-1 (publish/swap + swap-under-load), AC-2 (restart-once), AC-3 (retention gate
+ live lock gate), AC-4 (tombstones + concurrency), AC-5 (this), AC-6 (override).
Remaining plan tail: suite-driven recall gate (byte-identical proof is stronger)
and true disjoint-shard build-then-distribute (a separate feature).
