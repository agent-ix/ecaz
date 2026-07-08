# Review request — Task 165: multi-node CustomScan read path (loopback-validated)

**Branch:** `task-165-ec-distann-m3`. HEAD `fff3b5f1d`. This closes the read-path
half the reviewer flagged in 005-P1/006-P1: real multi-node scans now return
owner-owned SQL rows, not loopback-directory resolutions.

## What landed

`src/am/ec_distann/custom_scan.rs` — a CustomScan that **replaces the local index
scan** for a vector `ORDER BY <#> query LIMIT k` query when the roster is
multi-node:

1. **Planner** (`set_rel_pathlist_hook`): adds a `CustomPath` when an ec_distann
   index has the vector-ORDER-BY-LIMIT shape and `current_placement_directory()`
   is multi-node; the path wins over the (remote-incapable) index scan.
2. **Exec**: runs the shared `collect_distann_hits` search (packet's refactor);
   local hits carry a resolved ctid, remote hits carry INVALID. Remote hits are
   grouped by owning node and fetched via `ec_distann_materialize_row_payloads`
   (packet 010) over the pooled transport (`remote_materialize_row_payloads_batch`,
   reusing the warm 006-P3 sessions), then reconstructed into virtual tuples via
   `ReceiveFunctionCall`. Local hits are fetched from the local heap.
3. Registered in `_PG_init` alongside ec_spire.

Two exec bugs were caught by end-to-end validation (each crashed the backend)
and fixed — see the fix commit: (a) the scan slot is virtual, so local heap
fetches use a private buffer-heap slot then `ExecCopySlot` into the virtual scan
slot the projection reads; (b) a correlated LATERAL Param is bound per outer
row, so the query vector is evaluated per (re)scan, not at Begin.

## Evidence (`artifacts/`, release build, fresh loopback DB)

- `validate.log`: `EXPLAIN` picks `Custom Scan (EcDistannDistributedScan) on cs`;
  the **multi-node top-10 is byte-identical to single-node across 20 queries
  (0 mismatched ids)** — remote-owned hits (≈half the top-k under the 2-node
  hash placement) are shipped from the owner and reconstructed correctly.
- Fail-closed proof: pointing the remote roster node at a dead port makes the
  query error `[EC_INTERNAL] could not connect` — the remote path is genuinely
  exercised, not silently all-local.
- `setup.sql` / `validate.sql`: reproducible fixture + comparison.

## Honest scope

Loopback = both roster entries at the same instance (owner==coordinator
instance, remote ctids happen to be locally valid, but the rows travel the
shipping+reconstruction path, proven by the fail-closed check). The real
3-PG-instance fixture (Slice A) proves it across process boundaries, and the
3-worker `ecaz bench suite` gate (Slice D) scores recall. Those remain.

## Ask

Review the planner eligibility/costing, the exec local/remote split + slot
handling, and the tuple reconstruction. Not closing the task — fixture + gate
remain.
