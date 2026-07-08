# Review request — Task 164 M2: placement + epoch + expand-nodes endpoint

**Branch:** `task-164-ec-distann-m2` (stacked on `task-163-ec-distann-m1`; M2
consumes the deterministic stitched build — M1 not yet merged).
**Milestone:** M2 (two-node read path). This packet covers the landed
foundations; the remote transport + 2-node measurement follow.

## What landed (slices 1–2b)

1. **FR-078 hash placement** (`placement.rs`): `owning_node(vec_id) =
   placement_hash(vec_id) mod node_count`, versioned placement hash
   (`DISTANN_PLACEMENT_HASH_V1`, fmix64 with a distinct domain tag so it never
   aliases the identity hash), O(set-size) `group_by_owning_node`, and a
   topology-only `DistannPlacementDirectory` (roster + hash version + per-node
   counts; no per-record entries). Tests: AC-1 determinism, AC-3 within-epoch
   stability, AC-2 <10% imbalance across 3 nodes at 100k, grouping coverage,
   single-node degenerate.
2. **FR-082 epoch fingerprint (M2 subset)** (`epoch.rs`): 128-bit digest over
   the epoch's immutable identity (epoch, format version, placement version,
   roster order, build-time record-set fields). Tests: determinism, per-field
   sensitivity, roster-order sensitivity, length-prefix anti-aliasing, byte
   round-trip.
3. **Roster/epoch config** (`roster.rs`): `ec_distann.roster` /
   `.local_node_id` / `.epoch` GUCs + a pure, unit-tested `parse_roster`, and
   builders for the active `DistannPlacementDirectory` + local
   `DistannEpochIdentity`. (M3/FR-082 replaces GUC config with the persisted
   epoch manifest.)
4. **FR-079 `ec_distann_expand_nodes` endpoint** (`remote_endpoint.rs`): the
   `#[pg_extern]` SRF that runs the frozen-seam `LocalNodeExpander` for a batch
   of owned vec_ids and returns the wire rows in request order (no `heap_tid`).
   Plus `ec_distann_epoch_fingerprint(index)`. Validates the caller's epoch
   fingerprint (retriable on mismatch), enforces the FR-079 four-outcome table
   (placement error for non-owned ids; structural faults for owned-but-absent /
   vector-missing), and computes `exact_dist` from the co-placed heap (D11).

## Evidence

`artifacts/test-evidence.log` — 6 placement + 5 epoch + 5 roster pure tests,
and 3 endpoint pg_tests (happy path / epoch-mismatch / non-owned placement) all
green; clippy clean. No benchmark in this packet (M2's measured deliverable is
the H×RTT / two-node-vs-one-node latency, which lands with the transport slice).

## Design for the remaining M2 slices (for reviewer visibility)

- **2c — `RemoteNodeExpander` + transport + scan wiring.** A
  `DistannNodeExpander` that groups the beam batch by owning node, expands
  local ids in-process and remote ids via one pooled `ec_distann_expand_nodes`
  call per node (lifted SPIRE async libpq transport), and reassembles in
  request order — the FR-081 orchestration loop unchanged. The scan selects it
  when the roster has >1 node.
- **2d — two-node loopback fixture + TC-040/041 + H×RTT.** Loopback = one
  instance; each "remote" connection self-connects and sets its
  `local_node_id` to the target, so the full group→transport→endpoint→reassemble
  path runs on one instance. TC-040/041 assert 2-node top-k is identical to the
  single-node build on the same corpus/seed; the measured per-hop RTT is
  evaluated against the D4 baton reopen trigger (≥50% of multinode p50), and
  multinode scan defaults are set wide-BW/small-H per the G0 curves.

## Ask

Please review the placement hash, epoch-fingerprint identity coverage, the
FR-079 four-outcome handling in the endpoint, and the roster config surface.
Flag anything in the 2c/2d design before I build the transport. Do not close
this request.

## Notes

- Branch is stacked on M1; it will rebase onto M1 as that packet's reviewer
  feedback settles (already rebased once for the 2026-07-07-01 fixes).
- The endpoint reuses the head cache, which also builds the (endpoint-unused)
  head graph on first call — a noted M2 first-call cost, not a correctness issue.
