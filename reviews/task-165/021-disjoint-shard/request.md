# Review request — Task 165: true disjoint-shard storage, demonstrated

**Branch:** `task-165-ec-distann-m3`. Closes the last distribution gap — the
replicated-index model proved the read path; this proves it works with genuinely
**disjoint per-node storage** (each node holds only its owned shard).

## The model (build-global-then-prune, existing machinery)

A real disjoint deployment needs each node to hold only its owned records but
with the *global* graph adjacency. Achieved without new build machinery:

1. Build the identical deterministic global graph on every node (replicated).
2. On each node, delete the heap rows for vec_ids it does **not** own
   (`ec_distann_list_directory` × `ec_distann_owning_node`, new diagnostic SRFs),
   then `VACUUM` — ambulkdelete tombstones + reclaims those records.

Each node ends with only its owned records + co-placed heap rows; the global
adjacency lives in the owned records; head descent uses the (surviving) head
sample; hop rounds reach owners for their owned vec_ids.

## Evidence (`artifacts/distann-multinode-summary.log`, real 3× PG18)

```
disjoint_shard identical_after_prune=true per_node_rows[n1:2000->647 n2:2000->639 n3:2000->714]
```

- **Disjoint storage:** each node pruned to ~1/3 the corpus (647/639/714 of 2000);
  total across nodes = 2000 (a partition, not replicas).
- **Correctness:** the multi-node top-k result signature (md5 over 50 queries ×
  top-10) is **byte-identical** before and after pruning — the distributed read is
  unchanged by moving from replicated to disjoint storage. The drill fails the run
  if the signature changes.

## Status — M3 distribution complete

The multi-node read path is proven under BOTH the replicated substrate (packets
012/016/020) AND genuinely disjoint per-node storage (this). The
build-global-then-prune path uses only existing build + tombstone + VACUUM
machinery; a build-time direct owned-slice writer is a possible optimization, not
a correctness need.

## Ask

Review the two diagnostic SRFs (`ec_distann_list_directory`,
`ec_distann_owning_node`) and the disjoint drill's before/after-signature proof.
