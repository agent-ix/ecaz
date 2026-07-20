# Review request — Task 165 M3 slice 5: multinode scan materialization

**Branch:** `task-165-ec-distann-m3`. Completes the M2-carried materialization
tier — the multi-node read path now returns user-facing rows via `amgettuple`.

## What landed

`execute_distann_scan` now branches on the roster:
- **empty / single-node roster** → `LocalNodeExpander` (unchanged; 92 pg_tests
  green, no regression).
- **multi-node roster** → `RemoteNodeExpander` (group-by-owner → pooled parallel
  transport → FR-079 endpoint → position-reassembled), with the epoch
  fingerprint + roster spec computed once per scan.

**Remote-hit materialization**: remote responses carry `heap_tid = INVALID` (not
in the FR-079 wire contract). After orchestration, each remote hit's heap TID is
resolved from the local directory (loopback substrate: the coordinator holds the
full `vec_id → record` directory). A real multi-node deployment ships row data
from the owning node instead — there a remote hit's vec_id is absent locally and
the scan errors rather than mis-fetching (never silently wrong).

## Evidence (`artifacts/scan-compare.log`, release build)

2-node loopback `ORDER BY <#> LIMIT 10` on real 10k == single-node baseline:
`base_minus_two=0, two_minus_base=0, order_identical=t`. Verified
`ecaz_build_profile=release`.

## Ask

Review the scan roster-branch, the remote-hit materialization (and its
loopback-vs-real-multinode boundary), and confirm this unblocks the 50k
multinode recall bench. Not closing the request.
