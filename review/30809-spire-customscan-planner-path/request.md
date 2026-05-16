# Review Request: SPIRE CustomScan Planner Path

Code slice for Step 2 of the ADR-067 CustomScan pivot. This adds the first
planner path generation for `EcSpireDistributedScan` while keeping tuple
execution fail-closed until the executor wiring slice lands.

## Scope

- Extends the `set_rel_pathlist_hook` to discover base relations with an
  eligible remote-placement `ec_spire` index.
- Gates path generation on `ORDER BY ... LIMIT` planner state and the
  `ec_spire_custom_scan_index_eligibility` placement-directory signal.
- Adds a `CustomPath` with low LIMIT-based cost and carries the planner sort
  pathkeys so PostgreSQL can consume ordered CustomScan output without adding a
  separate sort.
- Implements `PlanCustomPath` enough to build a `CustomScan` plan node for
  `EXPLAIN` and stores the chosen index OID in `custom_private`.
- Keeps `BeginCustomScan` side-effect-free for plain `EXPLAIN`, and keeps
  `ExecCustomScan` fail-closed with the existing "not wired" error.
- Updates `ec_spire_custom_scan_status()` and Phase 11 task tracking to show
  planner path generation is now enabled while executor wiring remains open.

## Validation

- `cargo test customscan_explain --lib`
  - `test tests::pg_test_ec_spire_customscan_explain_remote_order_limit ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1608 filtered out`
- `cargo test custom_scan_status --lib`
  - `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_provider_name_and_disabled_execution ... ok`
  - `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`
- `cargo test customscan_eligibility --lib`
  - `test tests::pg_test_ec_spire_customscan_eligibility_no_active_epoch ... ok`
  - `test tests::pg_test_ec_spire_customscan_eligibility_no_available_remote ... ok`
- `cargo test custom_scan_index_eligibility_remote --lib`
  - `test tests::pg_test_ec_spire_custom_scan_index_eligibility_remote ... ok`
- `cargo fmt --check`
  - Passed; rustfmt still prints the repository's stable-toolchain warnings for
    nightly-only config keys.
- `git diff --check HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - Passed.

## Review Focus

- Confirm the planner gate is appropriate for this first path slice:
  base relation, `ORDER BY` pathkeys, finite `LIMIT`, and at least one
  available remote placement.
- Check the CustomPath cost and pathkey handling before executor wiring depends
  on ordered path selection.
- Check that `BeginCustomScan` being a no-op and `ExecCustomScan` being the
  fail-closed boundary is acceptable for plain EXPLAIN support.
- Note: this does not claim tuple delivery. The next slice must wire
  `ExecCustomScan` to `SpireRemoteFanoutExecutor` and return tuples directly.

## Artifacts

- `review/30809-spire-customscan-planner-path/artifacts/manifest.md`
- `review/30809-spire-customscan-planner-path/artifacts/cargo-test-customscan-explain.log`
- `review/30809-spire-customscan-planner-path/artifacts/cargo-test-custom-scan-status.log`
- `review/30809-spire-customscan-planner-path/artifacts/cargo-test-customscan-eligibility.log`
- `review/30809-spire-customscan-planner-path/artifacts/cargo-test-custom-scan-eligibility-remote.log`
- `review/30809-spire-customscan-planner-path/artifacts/cargo-fmt-check.log`
- `review/30809-spire-customscan-planner-path/artifacts/git-diff-check.log`
