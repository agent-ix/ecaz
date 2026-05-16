# Review Request: SPIRE Selected PID Placement Map

## Summary

Closes the Phase 12.7 row:

> Publish and inspect placement metadata that maps selected PIDs to remote
> nodes and local store IDs.

This adds `ec_spire_index_selected_pid_placement_snapshot(index_oid,
selected_pids)`, a narrow operator/debug diagnostic that reports one row per
requested PID:

- `active_epoch`
- `selection_ordinal`
- `pid`
- `node_id`
- `local_store_id`
- `store_relid`
- `placement_state`
- `object_version`
- `object_bytes`

The diagnostic uses the coordinator-fanout manifest loader so it can inspect
remote placements before fanout without tripping the local heap tuple delivery
guard. It validates the published manifest/directory pair and each requested
PID's object-version agreement before returning placement metadata.

## Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `git diff --check 5a066d05^ 5a066d05`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_selected_pid_placement_snapshot_sql`

## Reviewer Focus

- Confirm the function is appropriately scoped as a selected-PID placement
  diagnostic rather than a broader scan/dispatch decision report.
- Confirm using the coordinator-fanout manifest loader is the right boundary
  for inspecting synthetic/remote placements without requiring
  `custom_scan_tuple_delivery`.
- Confirm the tracker row closure is justified by the PG18 fixture, which
  rewrites one selected PID to remote node/local store `2` and verifies the
  returned selected-PID map includes both local and remote placements.
