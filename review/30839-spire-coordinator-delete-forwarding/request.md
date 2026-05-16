# SPIRE Coordinator DELETE Forwarding Primitive

## Scope

This packet adds the ADR-069 coordinator DELETE forwarding primitive and closes
the immediate `node_id = 0` gap from the prior UPDATE packet. It is
intentionally the reusable operation surface, not yet transparent
`DELETE ... WHERE pk = ...` planner/ModifyTable integration.

Changes:

- Adds remote endpoint `ec_spire_remote_delete_tuple_payload(index_oid,
  pk_column, pk_value)`.
  - It validates the primary-key column against the indexed heap relation.
  - It matches the remote row using the v1 canonical bigint primary-key bytes
    (`int8send(pk)::bytea`).
  - It returns the remote `deleted_count` to the coordinator.
- Adds coordinator helper
  `ec_spire_prepare_coordinator_delete_tuple_payload(index_oid, pk_column,
  pk_value)`.
  - It looks up `node_id` and `served_epoch` in `ec_spire_placement`.
  - It reuses the existing descriptor, conninfo-secret, epoch-window, timeout,
    and advisory-governance dispatch path.
  - It opens a remote transaction, performs the remote DELETE, prepares the
    remote transaction, removes the local placement-directory row, and relies on
    transaction callbacks to commit or roll back the prepared remote
    transaction with the local transaction outcome.
  - It fails closed unless the remote reports exactly one deleted row.
- Extends `ec_spire_forward_coordinator_update_tuple_payload` so placements
  resolved to `node_id = 0` update the coordinator heap locally instead of
  failing with `SPIRE placement resolved to local coordinator node_id 0`.
  This resolves reviewer P1 feedback from packet 30838.
- Adds focused PG18 coverage for both the prepared DELETE helper and the local
  UPDATE path.
- ADR-069 and the Phase 11 tracker now document the primitive, the local-node
  UPDATE behavior, and the remaining transparent DELETE hook gap.

## Validation

- `cargo test prepare_coordinator_delete --lib`
  - result: pass.
  - key line:
    `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - summary: `1 passed; 0 failed; 1640 filtered out`
- `cargo test forward_coordinator_update --lib`
  - result: pass.
  - key lines:
    `test tests::pg_test_ec_spire_forward_coordinator_update_local_sql ... ok`
    and
    `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
  - summary: `2 passed; 0 failed; 1639 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the DELETE helper's 2PC ordering is correct: remote prepared DELETE
  first, local placement removal in the same local transaction, then commit or
  rollback of the prepared remote transaction from xact callbacks.
- Confirm `remote_deleted_count == 1` is the right fail-closed behavior for the
  v1 DELETE primitive.
- Confirm the `node_id = 0` UPDATE branch is sufficient to resolve packet 30838
  P1 without weakening remote placement handling.
- Confirm it is appropriate to keep transparent DELETE integration open here,
  since remote-owned rows are absent from the coordinator heap and need the
  upcoming ModifyTable/view-hook surface rather than a normal row trigger.

## Artifacts

- `review/30839-spire-coordinator-delete-forwarding/artifacts/manifest.md`
- `review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-test-prepare-coordinator-delete-lib.log`
- `review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-test-forward-coordinator-update-lib.log`
- `review/30839-spire-coordinator-delete-forwarding/artifacts/cargo-fmt-check.log`
- `review/30839-spire-coordinator-delete-forwarding/artifacts/git-diff-check.log`
