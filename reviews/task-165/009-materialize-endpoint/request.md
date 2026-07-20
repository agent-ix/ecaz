# Review request — Task 165 005-P1: owner-side row-shipping materialize endpoint

**Branch:** `task-165-ec-distann-m3`. The data-path foundation for real
multi-node materialization (reviewer 005-P1 / 006-P1).

## Context (honest architecture)

005-P1 flagged that remote-hit materialization resolves each hit from the
coordinator's *own* directory, which only holds on the co-located/loopback
substrate. A genuinely distributed scan must get the row identity/data from the
**owning node**. This packet lands that owner-side data path.

## What landed

`ec_distann_materialize_rows(index_regclass, epoch_fingerprint, vec_ids)` — the
owning node returns, for each vec_id it owns, `(vec_id, heap_block, heap_offset,
is_tombstone)`: the heap identity of the co-placed row. It runs the same
FR-079/FR-082 preflight as `ec_distann_expand_nodes` — epoch fingerprint
validation (retriable mismatch), per-vec_id ownership (placement error),
owned-but-absent → `[EC_RECORD_MISSING]` — before any read. So a coordinator can
materialize remote hits by asking the owner, not by assuming a full local
directory.

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_materialize_rows_ships_heap_identity`: ships 2 owned rows with
valid, live heap ctids; a wrong fingerprint fails closed with the retriable
epoch-mismatch error. **102 distann pg_tests pass, 0 failed; clippy clean.**

## Honest remaining scope (the CustomScan)

This is the data-path half. *Returning* remote-owned rows through the executor
still needs a CustomScan: a remote ctid is not fetchable from the coordinator's
local heap, so the scan node must yield the owner-shipped identity/row directly
rather than routing through `amgettuple`'s local-heap fetch. On the co-located
substrate the shipped ctid is locally valid and the existing `amgettuple` path
already completes. The CustomScan integration + the 3-worker bench-suite exit
gate remain the primary M3 read-path work.

## Ask

Review the owner-side endpoint's preflight + the ctid shipping, and confirm the
CustomScan boundary. Not closing the request.
