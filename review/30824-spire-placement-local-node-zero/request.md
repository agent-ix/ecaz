# Review Request: SPIRE Placement Local Node Zero

## Scope

Feedback follow-up for the P2 in
`review/30817-spire-placement-directory-catalog/feedback/2026-05-11-001-reviewer.md`.

The reviewer noted that the new `ec_spire_placement.node_id CHECK (node_id > 0)`
excluded ADR-068 local node `0`. This packet relaxes the placement directory
constraint so coordinator-local shard ownership can be represented by the same
catalog surface as remote shard ownership.

This slice:

- Changes the bootstrap and upgrade SQL `ec_spire_placement.node_id` check from
  `node_id > 0` to `node_id >= 0`.
- Extends the direct placement-directory catalog SQL test to assert the
  constraint text and insert both a remote node and local node `0` row.
- Extends the placement batch registration test to preserve and store a local
  node `0` row.
- Updates the Phase 11 tracker to record the ADR-068 local-node fix.

This does not implement coordinator-routed INSERT, placement classification
write routing, or 2PC. It only corrects the catalog contract before those write
paths depend on it.

## Validation

- `cargo test ec_spire_placement --lib`
  - Passed: 2 tests.
- `cargo test placement_batch --lib`
  - Passed: 1 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm that `node_id >= 0` is the right catalog invariant for ADR-068 local
  node `0` plus positive remote node IDs.
- Confirm the tests cover both direct table writes and the batch registration
  helper path.
- Confirm no other placement-directory constraints need adjustment before the
  ADR-069 coordinator-routed write path starts.

## Artifacts

- `review/30824-spire-placement-local-node-zero/artifacts/manifest.md`
- `review/30824-spire-placement-local-node-zero/artifacts/cargo-test-ec-spire-placement-lib.log`
- `review/30824-spire-placement-local-node-zero/artifacts/cargo-test-placement-batch-lib.log`
- `review/30824-spire-placement-local-node-zero/artifacts/cargo-fmt-check.log`
- `review/30824-spire-placement-local-node-zero/artifacts/git-diff-check.log`
- `review/30824-spire-placement-local-node-zero/artifacts/git-diff-cached-check.log`
