# SPIRE Local DELETE Placement Handling

## Scope

This packet closes the coordinator-local half of the DELETE primitive. Placement
rows now allow `node_id = 0`, and UPDATE plus PK SELECT already handle that
case; DELETE still rejected it. This slice makes DELETE consistent.

Changes:

- Extracts a shared heap-delete helper used by both the remote DELETE endpoint
  and coordinator-local DELETE path.
- Updates `ec_spire_prepare_coordinator_delete_tuple_payload(...)` so
  `node_id = 0` placements:
  - delete the row directly from the coordinator heap;
  - remove the placement-directory row in the same local transaction;
  - skip remote dispatch and prepared transactions;
  - return `local_delete_applied` / `done`.
- Keeps remote-owned DELETE behavior unchanged: remote prepared DELETE plus
  local placement-directory delete.
- Updates ADR-069 and the Phase 11 tracker.

## Validation

- `cargo test prepare_coordinator_delete --lib`
  - result: pass.
  - key lines:
    `test tests::pg_test_ec_spire_prepare_coordinator_delete_local_sql ... ok`
    and
    `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - summary: `2 passed; 0 failed; 1643 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the local DELETE path should not use a remote prepared transaction.
- Confirm local heap delete plus placement-directory delete in the same local
  transaction is sufficient for `node_id = 0`.
- Confirm remote DELETE behavior was preserved.

## Artifacts

- `review/30842-spire-local-delete-placement/artifacts/manifest.md`
- `review/30842-spire-local-delete-placement/artifacts/cargo-test-prepare-coordinator-delete-lib.log`
- `review/30842-spire-local-delete-placement/artifacts/cargo-fmt-check.log`
- `review/30842-spire-local-delete-placement/artifacts/git-diff-check.log`
