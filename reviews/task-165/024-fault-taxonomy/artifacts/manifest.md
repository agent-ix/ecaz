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

Extends the TC-042 fault matrix from **6 → 12** drills, closing **all six**
NFR-020 taxonomy cases flagged in the packet-021 review (remote statement
timeout, hop-round failure mid-beam, missing node record, missing heap
row/co-placement drift, mid-insert failure, mid-delete failure). Three narrow
debug fault-injection GUCs were added (all off by default; each adds only a GUC
+ params field, no SQL): `ec_distann.debug_fail_hop_round`,
`ec_distann.debug_missing_node_record`, `ec_distann.debug_fail_insert`,
`ec_distann.debug_fail_tombstone_write`. The extension `.so` was swapped to the
debug build for each run and reverted to the shared release build afterward.

New cases:

-1. **missing_node_record** (FR-079 case c) — the local expander raises
    `OwnedRecordMissing` on its first expansion; the scan errors (tagged
    `missing node record`), never silently under-returns.
-1. **mid_insert_failure** (FR-083 fold path, TC-043) — `graph_insert_record`
    errors after staging the node + directory pages but before publishing
    metadata; on an isolated table, the aborting statement rolls the staged pages
    back and a post-fold scan is byte-identical to the pre-fold scan
    (`before_n=10 after_n=10 consistent=true`).
-1. **mid_delete_lost_tombstone_no_resurrect** (NFR-020 mid-delete) —
    `apply_record_writes` WAL-logs the tombstone flag flip then errors. The caller
    sees an error, and because the tombstone is a MONOTONIC set and PostgreSQL
    does not undo WAL-logged index-page writes on abort, the record is deleted and
    STAYS deleted (stable across re-reads, excluded from ANN scans) — the safe,
    non-resurrecting direction NFR-020 requires. (The initial drill asserted
    rollback-to-live; corrected after observing the monotonic-persist behavior.)

0. **hop_round_failure_mid_beam** — a narrow debug GUC
   (`ec_distann.debug_fail_hop_round`, default -1/off) makes the orchestrated
   search error at the start of a chosen 0-based hop round. The drill forces the
   search past round 0 (a high `top_k` bar defeats the round-0 early-exit) and
   injects a failure at round 1: the partial beam is discarded and the query
   ERRORs (verified: the error names `round 1`) — never a partial round-0
   frontier presented as complete. Extension `.so` was swapped to the debug build
   for this run and reverted to the shared release build afterward (the change
   adds no SQL, only a GUC + a `DistannOrchestrationParams` field).


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
- `fault_drill hop_round_failure_mid_beam pass=true`
- (stdout) `hop_round_failure_mid_beam DIAG errored=true mid_beam=true`
- `fault_drill missing_node_record pass=true`
- (stdout) `missing_node_record DIAG errored=true tagged=true`
- `fault_drill mid_delete_lost_tombstone_no_resurrect pass=true`
- (stdout) `mid_delete_lost_tombstone DIAG ... errored=true stable_tombstoned=true excluded_from_scan=true pass=true`
- `fault_drill mid_insert_failure_rolls_back pass=true`
- (stdout) `mid_insert_failure DIAG fold_errored=true before_n=10 after_n=10 consistent=true pass=true`
- `fault_drill missing_heap_row_co_placement_drift pass=true`
- (stdout) `co_placement_drift[owner=0] target_id=4 arm=correct_complete multi_n=10 single_n=10 pass=true`
- (stdout) `co_placement_drift[owner=2] target_id=3 arm=correct_complete multi_n=10 single_n=10 pass=true`
- `suite_recall_gate single=0.5000 multi=0.5000 delta=0.0000 pass=true`
- `qual_correctness single_n=10 multi_n=10 mismatch=0 pass=true`
- `recovery RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0 recovered=true`
- `disjoint_shard identical_after_prune=true per_node_rows[n1:2000->647 n2:2000->639 n3:2000->714]`
- GATE PASS: recall identical; 12 faults NFR-020-compliant; recovery clean.

## NFR-020 taxonomy coverage

All six of the reviewer's packet-021 named cases are now drilled and green, plus
the original six, for 12 total. This is the "100% drill pass across the taxonomy"
acceptance criterion for TC-042's multinode fault matrix. Note the
distributed-delete *routing* (coordinator → owner tombstone) is still a later M3
wire-up (dml.rs); the mid-delete drill exercises the owner endpoint
(`apply_record_writes`) directly, which is the FR-083 write contract's failure
surface.
