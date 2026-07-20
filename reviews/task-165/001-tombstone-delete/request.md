# Review request — Task 165 M3 slice 1: FR-083 tombstone delete (D10)

**Branch:** `task-165-ec-distann-m3` (stacked on M2). **Milestone:** M3 (first
DML slice); the full M3 (lifecycle + fault matrix + 50k bench) lands over the
following slices.

## What landed

New `dml` module implementing the ADR-085 D10 tombstone model — within a
Published epoch the only reclaim is a monotonic tombstone-flag set (FR-082
immutability):

- **`ambulkdelete`** now flags every record whose co-placed heap row the vacuum
  callback reports dead: an in-place, WAL-logged, same-length flag flip. Records,
  adjacency, and heap rows are all retained (next epoch build drops tombstones +
  repairs edges). Works for both identity modes — records store `heap_tid` and
  the callback is heap-TID-keyed, so no vec_id recompute at delete time.
- **`tombstone_by_vec_ids`** — the FR-083 write-endpoint "tombstone set"
  primitive: the coordinator routes a delete to the hash-owning node, which sets
  the flag on the record it owns. Monotone per vec_id; a vec_id absent from the
  directory is a structural fault (a delete must never silently miss).
- **`ec_distann_debug_tombstone`** SRF exposes it for in-transaction testing
  (VACUUM/`ambulkdelete` cannot run in pg_test's txn; the callback path is
  integration-tested against a committed DB in a later slice).

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_delete_tombstones_record` (FR-083-AC-1): tombstone 2 records by
vec_id → flags set on-disk (asserted via `read_node().tombstoned`), re-tombstone
is a monotone no-op, and the FR-081 scan excludes them (returns 6 of 8). clippy
clean.

## Remaining M3 slices (planned)

1. **Delta-buffer interim insert** (FR-083 / D5): `aminsert` spools to a bounded
   exact-scan delta buffer (needs a small incremental page-append helper — none
   exists post-build yet), merged into scan results with same-statement
   visibility, drained at epoch build. (FR-083-AC-3.)
2. **Write endpoint** `ec_distann_apply_record_writes` (FR-083): the remote
   counterpart to FR-079 — tombstone-set (above) + new-record append + back-edge
   amendment, epoch-fingerprint-validated. Multi-node tombstone routing.
3. **FR-082 full lifecycle**: 3-worker build/publish/retire, retirement gating +
   operator override.
4. **Fault-drill matrix** (TC-042 / NFR-020): the reused SPIRE cases +
   hop_round_failure_mid_beam, missing_node_record, placement_drift, mid-delete;
   every drill error-or-identical-to-baseline.
5. **50k multinode distinct_recall ≥ single-node − 0.001** via `ecaz bench
   suite`, release build.

## Ask

Review the tombstone mechanism (D10 monotone flag, both DML surfaces) and
confirm the slice plan. Not closing the request.
