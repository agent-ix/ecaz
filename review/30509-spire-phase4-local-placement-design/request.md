# Review Request: SPIRE Phase 4 Local Multi-Store Placement Design

## Checkpoint

- Code commit: `247b25d2`
  (`Document SPIRE local multi-store placement design`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local multi-store placement design

## Summary

Task 30 Phase 4 now has a durable design checkpoint in
`plan/design/spire-local-multistore-placement.md`.

The design records:

- a bounded relation-local `local_store_count` / `local_store_tablespaces`
  configuration surface;
- legacy/default single-store compatibility where store 0 remains the
  root/control index relation;
- dedicated AM-owned partition-store relations for multi-store indexes;
- root/control ownership of the active store set through a versioned store
  generation and store descriptors;
- placement-entry validation against the active store set, including
  `local_store_id`, `store_relid`, object TID, object bytes, and placement
  state;
- deterministic PID hash placement for leaf and routing objects, with deltas
  colocated with the parent leaf store;
- open and lock ordering across root/control and store relations;
- strict and degraded behavior when one local store is stale, skipped, or
  unavailable;
- a store-grouped scan fetch boundary that keeps candidate scoring close to
  partition-object bytes while deferring any benchmark-backed parallel
  multi-NVMe claim;
- placement and scan-placement diagnostic rows that report store identity,
  tablespace identity, object counts, bytes, candidate rows, scanned PID
  counts, and skipped placement state.

The Task 30 tracker now marks the design checkpoint complete and leaves the
implementation items open.

## Files

- `plan/design/spire-local-multistore-placement.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review whether the design is tight enough for the next implementation
slices:

- local store reloptions and root/control metadata codecs;
- store relation create/open/discovery helpers;
- deterministic hash placement planning;
- routing object, leaf object, and delta object write routing;
- store-grouped scan reads and placement diagnostics.

In particular, check whether the design makes the single-store compatibility
boundary explicit enough and whether the lock ordering, publish atomicity, and
degraded-store semantics are sufficient before relation-helper code lands.

## Validation

- `git diff --check`
- `git diff --cached --check`

Tests were not run because this is a documentation-only checkpoint.

## Notes

No measurement claims. The design explicitly defers one-store versus
multi-store benchmark claims until packet-local artifacts exist from a host
with multiple physical NVMe devices.

## Reviewer Follow-Up: 2026-05-06

- Follow-up code/docs commit: `e582739a`
  (`Address SPIRE phase 4 review feedback`)
- Kept this as an update to the existing Phase 4 design packet instead of
  opening another narrow helper packet.

Addressed reviewer feedback:

- `plan/status.md` now has a Task 30 SPIRE IVF row with Phase 4 status and
  remaining gates.
- `plan/design/spire-local-multistore-placement.md` now documents that
  `local_store_count` is fixed for a built index because changing it remaps
  existing object PIDs.
- `src/am/ec_spire/meta/local_store.rs` now documents the fixed store-count
  constraint next to `store_for_pid` and identifies `spire_pid_hash` as the
  durable SplitMix64-finalizer placement format.
- The scan grouping packet now collapses the leaf-only wrapper/type and
  surfaces filtered delta routes through diagnostics; see
  `review/30519-spire-scan-leaf-route-store-grouping/request.md`.

Still open from the design feedback:

- measured Task 30 recall/latency packet;
- store-relation DDL and relation-backed multi-store build publication;
- eventual multi-NVMe benchmark packet with packet-local raw artifacts.

Closed since this follow-up:

- `3d66fea4` adds an in-memory two-store write + scan-fetch fixture. It builds
  a hash-routed two-store partitioned draft, reads through the multi-store
  object-reader set, and proves scan candidates come from leaves in both local
  stores. Relation-backed auxiliary store DDL remains open.
- `7cb8298d` adds relation-backed scan store discovery/opening from the active
  placement directory, so scans dispatch reads by `(local_store_id,
  store_relid)` instead of assuming all object bytes live in the root/control
  index relation. Relation creation and relation-backed multi-store build
  publication remain open.
- `5b358440` adds PG18 coverage for a real two-relation write + scan-fetch
  path. The test uses two actual `ec_spire` index relations as root/control and
  auxiliary local store surfaces, writes objects across both relation files,
  publishes mixed-`store_relid` placements, and scans through
  placement-directed relation reads. User-facing auxiliary-store DDL remains
  open, but the second-store relation through-path is now lit.

Follow-up validation:

- `cargo test group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test collect_scan_placement_diagnostics --lib`
- `cargo pgrx test pg18 test_ec_spire_scan_placement_snapshot_sql`
- `cargo fmt`
- `git diff --check`
- `git diff --cached --check`

Additional validation for `3d66fea4`:

- `cargo test collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build --lib`
- `cargo test collect_quantized_routed_probe_candidates --lib`
- `cargo pgrx test pg18 test_ec_spire_populated_build_publishes_root_control`
- `cargo pgrx test pg18 test_ec_spire_relation_two_store_scan_roundtrip`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
