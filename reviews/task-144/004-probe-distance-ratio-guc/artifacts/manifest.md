# Task 144 Packet 004 Artifact Manifest

- head SHA: `a7b94ca0b2d2168a0972e1c2c4c28949c849488d`
- task bucket: `reviews/task-144/`
- packet path: `reviews/task-144/004-probe-distance-ratio-guc/`
- timestamp: `2026-07-05T09:31:47-07:00`
- slice: Task 144 Phase 2 default-off query-time probe distance-ratio pruning GUC plus ADR-084 surrogate decision.
- build profile: focused Rust unit tests only; no release benchmark matrix in this packet.
- isolated one-index-per-table vs shared-table: not applicable; no benchmark/index build was run.

## Artifacts

### `cargo-test-probe-distance-ratio.log`

- command:
  `script -q -c "cargo test -p ecaz probe_distance_ratio --no-default-features --features pg18" reviews/task-144/004-probe-distance-ratio-guc/artifacts/cargo-test-probe-distance-ratio.log`
- result:
  `COMMAND_EXIT_CODE="0"`
- key lines:
  - `running 2 tests`
  - `test am::ec_spire::options::tests::recursive_route_budget_carries_probe_distance_ratio ... ok`
  - `test am::ec_spire::scan::tests::route_recursive_routing_objects_to_leaf_routes_applies_probe_distance_ratio ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2264 filtered out; finished in 0.00s`

## Notes

This packet is a code-review checkpoint, not Task 144 closeout evidence. The full closeout still requires the reviewer-requested `ecaz bench suite` release A/B matrix with closure on/off and pruning on/off at 10k / 50k / 100k, including recall, latency, percent row-instances scanned, storage, per-query probed-list distributions, and recall tails.
