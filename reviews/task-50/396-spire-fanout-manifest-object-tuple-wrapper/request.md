# SPIRE Fanout Manifest Object Tuple Wrapper

## Scope

This packet covers commit `da853aa64`, which threads
`load_relation_epoch_manifests_for_coordinator_fanout` through
`SpireLiveIndexRelation` instead of accepting a raw `pg_sys::Relation`.

Touched code:

- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
- `src/am/ec_spire/coordinator/snapshots.rs`
- `src/am/ec_spire/coordinator/debug.rs`
- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs`
- `src/am/ec_spire/coordinator/remote_candidates/fanout.rs`
- `src/am/ec_spire/coordinator/remote_candidates/fault_matrix.rs`

## Result

- Replaced the fanout manifest loader's direct `page::read_object_tuple`
  unsafe block with `SpireLiveIndexRelation::object_tuple`.
- Removed raw relation conversion locals from callers that already had a typed
  live-index wrapper.
- Current `src` unsafe ledger count after this slice: `1119`.

## Validation

See `artifacts/manifest.md`.
