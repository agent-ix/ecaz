# Review request — Task 165: AC-4 concurrency drill on the real 3-instance fixture

**Branch:** `task-165-ec-distann-m3`. Adds the FR-082-AC-4 *concurrent* half to the
`ecaz dev distann-multicluster` fixture: many multi-node scans running
concurrently with a background inserter mutating the coordinator's table.

## What landed

`concurrency_drill` (crates/ecaz-cli/.../distann_multicluster.rs): 4 concurrent
multi-node scan loops (12 iters each) on the coordinator + 1 background inserter,
all fired at once via tokio. Each scan must complete without error — a
half-applied/torn read under concurrent mutation would surface as an error. The
drill fails the run if any session errors.

## Evidence (`artifacts/distann-multinode-summary.log`, real 3× PG18, v4 .so)

```
RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0
concurrency_scan_under_insert pass=true
fault_drill simulated_network_partition pass=true
fault_drill epoch_bump_no_false_reject pass=true
fault_drill remote_content_divergence pass=true
fault_drill missing_or_reindexed_remote_index pass=true
fault_drill remote_backend_termination pass=true
fault_drill placement_drift pass=true
recovery ... mismatched_ids=0 recovered=true
```

## Remaining tail

AC-4's "never a half-applied back-edge amendment" is upheld by the M5 insert's
per-record write atomicity; this drill exercises it under real concurrent load.
Epoch-swap-under-load (needs the scan to read the active epoch from the persisted
manifest, not the GUC), the AC-5 cross-epoch frozen tier (D11 reversal), and the
live in-flight counter (shared-memory infra) remain — see packet 014.
