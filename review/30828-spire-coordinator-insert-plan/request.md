# Review Request: SPIRE Coordinator Insert Planning Primitive

## Scope

First narrow ADR-069 coordinator-routed INSERT slice.

This packet adds the side-effect-free planning primitive that the later remote
2PC INSERT executor will use before dispatching to a remote shard. It does not
yet forward the row or write the placement directory.

This slice:

- Adds `ec_spire_plan_coordinator_insert(index_oid, pk_value, embedding,
  source_identity)`.
- Validates `pk_value` is non-empty.
- Validates `source_identity` is exactly 16 bytes.
- Calls the same active-epoch classifier used by
  `ec_spire_classify_centroid(...)`.
- Returns `(index_oid, pk_value, node_id, centroid_id, served_epoch,
  source_identity)`, matching the placement tuple fields the future 2PC INSERT
  path will persist after remote prepare succeeds.
- Documents the primitive in ADR-069 and marks the classification/planning
  sub-slice in the Phase 11 tracker.

This intentionally avoids side effects. It is not the final transparent
`INSERT INTO tbl ...` path and does not claim remote dispatch, prepared
transactions, placement-directory mutation, or RETURNING support.

## Validation

- `cargo test plan_coordinator_insert --lib`
  - Passed: 3 PG18 tests.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm this side-effect-free primitive is the right first boundary for
  coordinator-routed INSERT before remote 2PC dispatch.
- Confirm the returned fields match the `ec_spire_placement` tuple contract and
  preserve the active classifier's `node_id`, routing leaf pid, and epoch.
- Confirm validation should stay limited to canonical PK bytes and 16-byte
  source identity here, leaving duplicate placement rows and remote dispatch
  failures to the later mutating INSERT path.

## Artifacts

- `review/30828-spire-coordinator-insert-plan/artifacts/manifest.md`
- `review/30828-spire-coordinator-insert-plan/artifacts/cargo-test-plan-coordinator-insert-lib.log`
- `review/30828-spire-coordinator-insert-plan/artifacts/cargo-fmt-check.log`
- `review/30828-spire-coordinator-insert-plan/artifacts/git-diff-check.log`
- `review/30828-spire-coordinator-insert-plan/artifacts/git-diff-cached-check.log`
