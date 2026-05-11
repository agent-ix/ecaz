# Review Request: SPIRE CustomScan Eligibility Planner Readiness

Code slice for reviewer feedback on packet `30806`. This keeps planner path
generation disabled, but makes the SQL-visible CustomScan eligibility surface
cheap and precise enough for the next planner hook slice.

## Scope

- Replaces the eligibility function's full coordinator-fanout manifest load
  with a narrow placement-directory object-tuple read.
- Keeps the local-only index AM path unchanged.
- Adds planner-gating signals to
  `ec_spire_custom_scan_index_eligibility(...)`:
  `remote_available_node_count`, `remote_unavailable_placement_count`, and
  `all_remote_placements_available`.
- Preserves `eligible_for_custom_scan = true` when at least one remote
  placement is available, while exposing partial/unavailable remote placement
  state for later costing and path gating.
- Adds comments documenting the intentionally inert CustomPath methods and hook
  body until the path-generation slice lands.
- Updates the Phase 11 task file for packet `30808` and removes stale
  AM-boundary wording from the CustomScan pivot section.

## Validation

- `cargo test customscan_eligibility --lib`
  - `test tests::pg_test_ec_spire_customscan_eligibility_no_active_epoch ... ok`
  - `test tests::pg_test_ec_spire_customscan_eligibility_no_available_remote ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1606 filtered out`
- `cargo test custom_scan_index_eligibility_remote --lib`
  - `test tests::pg_test_ec_spire_custom_scan_index_eligibility_remote ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1607 filtered out`
- `cargo fmt --check`
  - Passed; rustfmt still prints the repository's stable-toolchain warnings for
    nightly-only config keys.
- `git diff --check HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - Passed.

## Review Focus

- Confirm the narrow placement-directory read is the right planner-readiness
  helper and does not need epoch/object manifest validation for this gate.
- Check the semantics of `eligible_for_custom_scan`, especially partial
  availability: at least one available remote placement is a candidate, while
  `all_remote_placements_available` exposes whether costing should be
  conservative.
- Confirm the added SQL columns are the right planner-facing signals before
  actual CustomPath generation depends on this surface.

## Artifacts

- `review/30808-spire-customscan-eligibility-planner-readiness/artifacts/manifest.md`
- `review/30808-spire-customscan-eligibility-planner-readiness/artifacts/cargo-test-customscan-eligibility.log`
- `review/30808-spire-customscan-eligibility-planner-readiness/artifacts/cargo-test-custom-scan-eligibility-remote.log`
- `review/30808-spire-customscan-eligibility-planner-readiness/artifacts/cargo-fmt-check.log`
- `review/30808-spire-customscan-eligibility-planner-readiness/artifacts/git-diff-check.log`
