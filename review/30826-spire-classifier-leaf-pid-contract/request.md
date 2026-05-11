# Review Request: SPIRE Classifier Leaf-Pid Contract

## Scope

Feedback follow-up for the P2 items in
`review/30818-spire-classify-centroid-helper/feedback/2026-05-11-001-reviewer.md`.

The reviewer accepted `ec_spire_classify_centroid(...)` but called out that
`centroid_id` was really the selected routing leaf pid, and that the classifier
needed stronger recursive evidence plus a traversal-depth bound before
coordinator-routed writes consume it.

This slice:

- Documents in ADR-069 that `centroid_id` is the active-epoch routing leaf pid,
  scoped to `(index_oid, served_epoch)`, not a stable semantic centroid across
  retraining/rebalance.
- Adds a classifier traversal-depth bound derived from the root routing level.
- Adds a recursive PG18 classifier fixture with `nlists = 4` and
  `recursive_fanout = 2`.
- The recursive fixture independently scores root children and leaf children
  through `ec_spire_index_routing_centroid_snapshot(...)`, rewrites the selected
  leaf placement to `node_id = 9`, and asserts
  `ec_spire_classify_centroid(...)` returns that node, leaf pid, and active
  epoch.
- Updates the Phase 11 tracker.

This does not rename the existing `centroid_id` column. It pins the current name
as a v1 compatibility contract and documents the semantics explicitly.

## Validation

- `cargo test classify_centroid --lib`
  - Passed: 2 tests.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm that documenting `centroid_id` as active-epoch routing leaf pid is
  sufficient for v1 instead of renaming the catalog column.
- Confirm the recursive fixture covers the classifier's multi-level traversal
  path and the `parent.header.level == 1` leaf-parent gate.
- Confirm the root-level traversal bound is the right defensive limit for
  malformed routing hierarchies.

## Artifacts

- `review/30826-spire-classifier-leaf-pid-contract/artifacts/manifest.md`
- `review/30826-spire-classifier-leaf-pid-contract/artifacts/cargo-test-classify-centroid-lib.log`
- `review/30826-spire-classifier-leaf-pid-contract/artifacts/cargo-fmt-check.log`
- `review/30826-spire-classifier-leaf-pid-contract/artifacts/git-diff-check.log`
- `review/30826-spire-classifier-leaf-pid-contract/artifacts/git-diff-cached-check.log`
