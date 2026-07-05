# Review Request: SPIRE Delta Reuse Coverage

Code checkpoint: `f86fdcca` (`Cover SPIRE delta reuse in remote candidates`)

## Scope

- Advances Phase 12.8 by documenting that selected-leaf delta decoding is
  shared between delete suppression and delta-insert candidate scoring.
- Documents that the same selected-leaf collector backs remote local-heap and
  tuple-payload candidate resolution before origin-node payload lookup.
- Extends `test_ec_spire_remote_search_local_heap_resolution_plan` to insert a
  post-build delta row, recompute the active epoch and selected leaves, then
  assert `ec_spire_remote_search_local_heap_candidates(...)` returns all three
  rows with one delta object and valid local locators.
- Keeps the existing unit coverage that asserts
  `load_delta_rows_for_routes(...)` reads each selected delta object once.
- Marks the Phase 12.8 delta-decode reuse row complete in the Phase 12 tracker.

## Validation

- `git diff --check f86fdcca^ f86fdcca`
- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 load_delta_rows_for_routes_reads_each_delta_object_once --lib`
- `cargo pgrx test pg18 test_ec_spire_remote_search_local_heap_resolution_plan`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and key result lines.

## Review Focus

- Confirm the remote candidate fixture exercises the intended post-build delta
  path without depending on tuple-payload compatibility work that belongs to
  Phase 12.2.
- Confirm the design note accurately describes the delta-row reuse boundary:
  selected delta rows are loaded once per selected delta route set, then reused
  for delete suppression and candidate scoring.
