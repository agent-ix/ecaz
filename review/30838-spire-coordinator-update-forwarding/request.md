# SPIRE Coordinator UPDATE Forwarding Primitive

## Scope

This packet adds the first non-embedding UPDATE implementation surface for the
ADR-069 write path. It is intentionally the forwarding primitive, not yet the
transparent `UPDATE ... WHERE pk = ...` planner/ModifyTable hook.

Changes:

- Adds remote endpoint `ec_spire_remote_update_tuple_payload(index_oid,
  pk_column, pk_value, row_payload, updated_columns)`.
  - It validates the primary-key column and explicit update columns against the
    indexed heap relation.
  - It rejects primary-key updates.
  - It matches the remote row using the v1 canonical bigint primary-key bytes
    (`int8send(pk)::bytea`) and updates only the requested payload columns.
- Adds coordinator helper
  `ec_spire_forward_coordinator_update_tuple_payload(index_oid, pk_column,
  pk_value, row_payload, updated_columns)`.
  - It looks up `node_id` and `served_epoch` in `ec_spire_placement`.
  - It reuses the existing remote descriptor, conninfo-secret, epoch-window,
    timeout, and advisory-governance dispatch path.
  - It forwards the UPDATE to the owning remote without two-phase commit,
    matching ADR-069's non-embedding UPDATE contract.
- Adds focused PG18 coverage proving a `title` update routes by placement and
  mutates the owning remote row.
- ADR-069 and the Phase 11 tracker now document the primitive and explicitly
  keep transparent UPDATE hook integration open.

## Validation

- `cargo test forward_coordinator_update --lib`
  - result: pass.
  - key line: `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
  - summary: `1 passed; 0 failed; 1638 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the split is appropriate: this packet lands the reusable UPDATE
  forwarding operation while leaving transparent `UPDATE` routing to a follow-up
  because remote-owned rows are absent from the coordinator heap.
- Confirm the helper correctly treats `ec_spire_placement` as the source of
  truth for `pk_value -> node_id`.
- Confirm no two-phase commit is needed for this non-embedding UPDATE helper
  because no coordinator-side state changes after the placement lookup.

## Artifacts

- `review/30838-spire-coordinator-update-forwarding/artifacts/manifest.md`
- `review/30838-spire-coordinator-update-forwarding/artifacts/cargo-test-forward-coordinator-update-lib.log`
- `review/30838-spire-coordinator-update-forwarding/artifacts/cargo-fmt-check.log`
- `review/30838-spire-coordinator-update-forwarding/artifacts/git-diff-check.log`
