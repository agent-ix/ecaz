# Packet 024 — NFR-020 fault-taxonomy extension (artifacts manifest)

- head SHA: f5e40831596bef5402afcf8930a5e2a6d669e772
- task bucket / packet: reviews/task-165/024-fault-taxonomy
- surface: real 3× PG18 fixture (`ecaz dev distann-multicluster`), replicated
  deterministic corpus, installed release `.so`
- fixture params: nodes=3, rows=2000, dim=16, graph_degree=32, queries=50, top_k=10
- isolation: replicated (one identical global graph per node) + destructive
  co-placement/disjoint drills that self-recover via deterministic re-setup
- timestamp: 2026-07-09T17:01:28Z

## Command

```
ecaz dev distann-multicluster local-multinode-pg18 \
  --nodes 3 --rows 2000 --dim 16 --queries 50 --top-k 10 \
  --artifact-dir reviews/task-165/024-fault-taxonomy/artifacts \
  --log-file reviews/task-165/024-fault-taxonomy/artifacts/fixture-run.log
```

## What changed vs packet 023

Extends the TC-042 fault matrix from **6 → 8** drills, closing two NFR-020
taxonomy gaps flagged in the packet-021 review:

1. **remote_statement_timeout** — inject `options=-cstatement_timeout=1` (1 ms)
   into a single owner's roster conninfo; that owner's expand statement is
   cancelled server-side and the coordinator surfaces the remote error rather
   than a partial/complete-looking result. Pass = query errored (fail closed).

2. **missing_heap_row_co_placement_drift** — delete a live record's heap row on
   **every** node (the index/directory record survives on each ⇒ cluster-wide
   dangling record / missing co-placed vector), run over **both** a
   coordinator-owned (owner=0) and a remote-owned (owner=nodes−1) target.
   Asserts the NFR-020 disjunction (`NFR-020:23-26`): the multinode scan SHALL
   either raise an error OR return a correct complete result — proven by
   equality to a single-node scan over the same deleted corpus with the target
   excluded — never a partial/stale result presented as complete.

### Why the drift drill needed reshaping (evidence, not assumption)

- A single-owner delete was **masked** by the other replicas: DIAG showed
  `target_id=3 errored=false target_in_result=true` — the coordinator served its
  own local copy. Ownership partitions materialization, not the local graph
  search, so the drill must delete cluster-wide to produce real drift.
- Both ownership arms then returned `arm=correct_complete` (`multi_n==single_n`,
  target excluded): the read path skips the MVCC-invisible co-placed row
  consistently on local and remote owners. The FR-079 case-(d) `EC_VECTOR_MISSING`
  error path fires only under genuine *unreadable* corruption (not injectable via
  a clean SQL DELETE); that arm is covered by single-node pg_test TC-040.

## Key result lines (`artifacts/distann-multinode-summary.log`)

- `RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0`
- `fault_drill remote_statement_timeout pass=true`
- `fault_drill missing_heap_row_co_placement_drift pass=true`
- (stdout) `co_placement_drift[owner=0] target_id=4 arm=correct_complete multi_n=10 single_n=10 pass=true`
- (stdout) `co_placement_drift[owner=2] target_id=3 arm=correct_complete multi_n=10 single_n=10 pass=true`
- `suite_recall_gate single=0.5000 multi=0.5000 delta=0.0000 pass=true`
- `qual_correctness single_n=10 multi_n=10 mismatch=0 pass=true`
- `recovery RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0 recovered=true`
- `disjoint_shard identical_after_prune=true per_node_rows[n1:2000->647 n2:2000->639 n3:2000->714]`
- GATE PASS: recall identical; 8 faults NFR-020-compliant; recovery clean.

## Remaining NFR-020 taxonomy gaps (not in this packet)

Still open from the reviewer's list: `hop_round_failure_mid_beam`,
`missing_node_record`, and `mid-insert failure` (FR-083 insert path).
`mid-delete failure` (lost remote tombstone → row must not resurrect) is
partially exercised by the co-placement drift + AC-5 frozen-vector drills but
not yet as a dedicated lost-tombstone-write injection.
