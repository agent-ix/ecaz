# Task 144 Packet 005 Artifact Manifest

- head SHA: `a04cb85cb4e18eb79391ab792c30b9ff1b85f450`
- task bucket: `reviews/task-144/`
- packet path: `reviews/task-144/005-closure-epsilon-reloption/`
- timestamp: `2026-07-05T09:44:07-07:00`
- slice: Task 144 Phase 1 default-off `closure_epsilon` reloption and build-side closure assignment routing.
- build profile: focused Rust unit tests only; no release benchmark matrix in this packet.
- isolated one-index-per-table vs shared-table: not applicable; no benchmark/index build was run.

## Artifacts

### `cargo-test-closure.log`

- command:
  `script -q -c "cargo test -p ecaz closure --no-default-features --features pg18" reviews/task-144/005-closure-epsilon-reloption/artifacts/cargo-test-closure.log`
- result:
  `COMMAND_EXIT_CODE="0"`
- key lines:
  - `running 2 tests`
  - `test am::ec_spire::options::tests::closure_epsilon_reloption_accepts_default_off_ratio_band ... ok`
  - `test am::ec_spire::build::tests::single_level_route_map_plans_closure_replica_pids_by_distance_ratio ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2266 filtered out; finished in 0.00s`

## Notes

This packet lands code-review substrate only. It does not claim Task 144 closeout, storage impact, or recall/latency behavior. Closeout still requires the release `ecaz bench suite` A/B matrix with closure on/off and pruning on/off at 10k / 50k / 100k, including recall, latency, percent row-instances scanned, storage, per-query probed-list distributions, and recall tails.
