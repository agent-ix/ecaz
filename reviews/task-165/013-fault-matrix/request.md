# Review request — Task 165: TC-042 fault matrix on the real 3-instance fixture (Slice C core)

**Branch:** `task-165-ec-distann-m3`. HEAD (fault-matrix commit). Extends the
`ecaz dev distann-multicluster` fixture (packet 012) with a real cross-process
fault matrix asserting NFR-020's **error-or-identical-to-baseline** bar.

## What landed

The fixture now runs, after the recall gate, a fault matrix against the real
3-instance deployment, each fault classified per NFR-020:

| drill | injection | NFR-020 outcome |
|---|---|---|
| `simulated_network_partition` | one owner roster entry → dead port | ERROR (fail closed) |
| `epoch_bump_no_false_reject` | coordinator `ec_distann.epoch` bumped | IDENTICAL (content-based fingerprint + propagated epoch ⇒ no false reject) |
| `remote_content_divergence` | rebuild owner index at a different `graph_degree` → fingerprint mismatch | ERROR (fail closed) |
| `missing_or_reindexed_remote_index` | `DROP INDEX` on an owner | ERROR (fail closed) |
| `remote_backend_termination` | `pg_ctl stop` an owner instance | ERROR (fail closed) |
| `placement_drift` | coordinator `local_node_id` absent from the roster | ERROR (no local node ⇒ fail closed) |
| `recovery` | all faults cleared | IDENTICAL to baseline |

**Deliberately not drilled — base-table DELETE.** An early attempt drilled a raw
`DELETE` as "mid-delete"; it produced `[EC_VECTOR_MISSING]` because ec_distann's
co-placed exact rerank needs the epoch's frozen vector, which a base DELETE
removes. That violates the FR-082 Published-epoch model (deletion is a monotonic
**tombstone-flag** set via FR-083's `ec_distann_apply_record_writes`, which keeps
the vector) — and is exactly the hazard FR-082-AC-5's epoch-owned frozen snapshot
exists to prevent. A correct distributed-tombstone drill needs per-node ownership
bucketing (an `owning_node` SQL surface) and is a follow-up.

## Evidence (`artifacts/distann-multinode-summary.log`, real 3× PG18)

```
RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0
fault_drill simulated_network_partition pass=true
fault_drill epoch_bump_no_false_reject pass=true
fault_drill remote_content_divergence pass=true
fault_drill missing_or_reindexed_remote_index pass=true
fault_drill remote_backend_termination pass=true
recovery RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0 recovered=true
```

The run **fails hard** (non-zero exit) if any drill is not fail-closed, the recall
gate mismatches, or recovery diverges — so this is a real gate, not a report.

## Honest remaining scope (Slice C tail)

Representative NFR-020 coverage across the transport / fingerprint / index /
backend / recovery axes is proven. The remaining task-file taxonomy items —
`remote_statement_timeout`, `hop_round_failure_mid_beam`, `missing_node_record`,
`placement_drift`, `mid-delete` (FR-083 distributed) — plus the full FR-082
build/publish/retire lifecycle + epoch-swap-under-load, need deeper injection
(GUC plumbing, catalog corruption, concurrent load) and are follow-ups on this
same fixture.

## Ask

Review the fault classifications (especially that `epoch_bump_no_false_reject`
is correctly the *identical* arm, not an error) and the fixture's hard-gate
assertions.
