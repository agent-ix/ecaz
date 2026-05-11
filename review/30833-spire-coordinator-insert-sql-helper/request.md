# Review Request: SPIRE Coordinator Insert SQL Helper

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30832`.

This packet adds the production SQL helper that composes the reviewed INSERT
pieces before the transparent DML hook exists. The helper classifies the row,
prepares the typed remote tuple-payload INSERT, and stages the
placement-directory row in the coordinator transaction.

This slice:

- Adds `ec_spire_prepare_coordinator_insert_tuple_payload(index_oid, pk_value,
  embedding, source_identity, row_payload, requested_columns)`.
- Validates canonical primary-key bytes and ADR-063 source identity length.
- Reuses the active SPIRE classifier to choose `node_id`, `centroid_id`, and
  `served_epoch`.
- Reuses packet `30832`'s remote tuple-payload prepare path, including remote
  descriptor dispatch, libpq transport, remote `PREPARE TRANSACTION`, and
  coordinator xact callbacks.
- Inserts the placement-directory row only after remote prepare succeeds.
- Returns the selected placement tuple, prepared GID, remote prepare flags,
  placement-staged flag, status, and next step.
- Adds PG18 coverage for the composed helper with loopback remote prepared
  transaction and placement staging.
- Documents the helper in ADR-069 and marks the Phase 11 tracker sub-slice.

This intentionally does not install the transparent coordinator INSERT hook or
prove final read-after-insert through CustomScan. The hook still needs to
construct the canonical PK bytes, ADR-063 source identity, JSON tuple payload,
and explicit column list from the executor tuple before calling this operation.

## Validation

- `cargo test prepare_coordinator_insert_tuple_payload --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm this helper is a useful boundary for the eventual transparent
  coordinator INSERT hook.
- Confirm remote prepare happens before local placement staging.
- Confirm local placement staging failure would abort the coordinator
  transaction and let the registered abort callback roll back the prepared
  remote transaction.
- Confirm the SQL surface does not overclaim transparent DML: callers still
  provide canonical PK bytes, source identity, row JSON, and column list.

## Artifacts

- `review/30833-spire-coordinator-insert-sql-helper/artifacts/manifest.md`
- `review/30833-spire-coordinator-insert-sql-helper/artifacts/cargo-test-prepare-coordinator-insert-tuple-payload-lib.log`
- `review/30833-spire-coordinator-insert-sql-helper/artifacts/cargo-fmt-check.log`
- `review/30833-spire-coordinator-insert-sql-helper/artifacts/git-diff-check.log`
- `review/30833-spire-coordinator-insert-sql-helper/artifacts/git-diff-cached-check.log`
