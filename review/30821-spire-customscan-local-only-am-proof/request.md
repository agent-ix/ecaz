# Review Request: SPIRE CustomScan Local-Only AM Proof

## Scope

Follow-up slice for the ADR-067 CustomScan read path. This closes the tracker
item that local-only `ec_spire` index scans remain on the index AM path while
the CustomScan planner path is reserved for active remote placements.

- Add PG18 plan coverage:
  `test_ec_spire_customscan_does_not_replace_local_only_index_plan`.
- The fixture creates a local-only `ec_spire` index, disables seqscan, leaves
  indexscan enabled, and runs
  `SELECT id ... ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1`.
- It asserts the plan does not contain
  `Custom Scan (EcSpireDistributedScan)`.
- It asserts the plan still contains `Index Scan`, proving the existing
  local-only index AM planning path remains available.
- Update the Phase 11 tracker to mark the local-only AM preservation proof
  complete.

This does not change planner path generation, costing, executor behavior, or
ADR-069 write-path work.

## Validation

- `cargo test customscan_does_not_replace_local_only_index_plan --lib`
  - Passed: 1 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Check whether the plan assertion is strong enough for "AM path preserved":
  it intentionally checks the user-visible plan shape rather than internal
  planner hooks.
- Check that disabling only seqscan is the right pressure for this local-only
  query shape; indexscan remains enabled to preserve the AM path.
- Confirm the tracker update does not overstate the remaining open production
  cost model or ADR-069 write work.

## Artifacts

- `review/30821-spire-customscan-local-only-am-proof/artifacts/manifest.md`
- `review/30821-spire-customscan-local-only-am-proof/artifacts/cargo-test-customscan-local-only-lib.log`
- `review/30821-spire-customscan-local-only-am-proof/artifacts/cargo-fmt-check.log`
- `review/30821-spire-customscan-local-only-am-proof/artifacts/git-diff-check.log`
- `review/30821-spire-customscan-local-only-am-proof/artifacts/git-diff-cached-check.log`
