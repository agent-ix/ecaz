# Task 39 SpireLocalObjectStoreSet non-leaf coverage

## Summary

Pins the insert/read delegation paths through
`SpireLocalObjectStoreSet`'s `SpireObjectReader` implementation for
every non-leaf object kind:

- `insert_routing_object` + `read_routing_object`
- `insert_delta_object` + `read_delta_object`
- `insert_top_graph_object` + `read_top_graph_object`
- `read_object_header` (verified on the routing placement)

The pre-existing
`local_object_store_set_routes_by_pid_and_reads_back_objects` test
covered only the leaf-V2 path, so a mis-routed `store_for_placement`
would have been silent for the other three object kinds — a gap in
`src/am/ec_spire/storage/local_store_set.rs` coverage.

## Code under review

- Commit: `6afcc6911427e4f84437b452a23edfbe65b669df`
- Changed file: `hardening/careful/src/spire.rs`

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib
  local_object_store_set`: 4 passed (up from 3 prior). Artifact:
  `artifacts/store-set-focused-tests.log`.
- Full careful test suite: 455 passed.

## Notes

- The new test lives in `hardening/careful/src/spire.rs` next to the
  other `local_object_store_set_*` tests, since the helpers
  (`routing_children`, `leaf_v2_assignment`) and the
  `SpirePartitionObjectKind` import live there.
- Delta assignment requires `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT`;
  `leaf_v2_assignment` deliberately produces a non-delta row, so the
  delta sub-case constructs `SpireLeafAssignmentRow` inline with the
  correct flag combination.
