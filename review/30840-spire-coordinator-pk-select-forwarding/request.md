# SPIRE Coordinator PK SELECT Forwarding Primitive

## Scope

This packet adds the ADR-069 PK-keyed read forwarding primitive. It is the
dispatch operation that a later transparent `SELECT ... WHERE pk = ...`
planner/view-hook front door should call.

Changes:

- Adds remote endpoint `ec_spire_remote_select_tuple_payload(index_oid,
  pk_column, pk_value, requested_columns)`.
  - It validates the primary-key column and requested projection columns
    against the indexed heap relation.
  - It matches the remote row using the v1 canonical bigint primary-key bytes
    (`int8send(pk)::bytea`).
  - It returns `selected_count`, requested payload-column count, and a JSON
    tuple payload for the projected columns.
- Adds coordinator helper
  `ec_spire_forward_coordinator_select_tuple_payload(index_oid, pk_column,
  pk_value, requested_columns)`.
  - It looks up `node_id` and `served_epoch` in `ec_spire_placement`.
  - `node_id = 0` placements are served directly from the coordinator heap.
  - Remote placements reuse the descriptor, conninfo-secret, epoch-window,
    timeout, and advisory-governance dispatch path.
- Adds focused PG18 coverage for both remote and local placement paths.
- Updates ADR-069 and the Phase 11 tracker to mark the PK-read primitive done
  while keeping transparent PK SELECT integration open.

## Validation

- `cargo test forward_coordinator_select --lib`
  - result: pass.
  - key lines:
    `test tests::pg_test_ec_spire_forward_coordinator_select_local_sql ... ok`
    and
    `test tests::pg_test_ec_spire_forward_coordinator_select_tuple_payload_sql ... ok`
  - summary: `2 passed; 0 failed; 1641 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the PK-read helper should return JSON tuple payload text at this
  primitive layer, leaving typed tuple projection to the eventual transparent
  front door.
- Confirm placement lookup and `node_id = 0` local handling match the UPDATE
  helper semantics.
- Confirm the remote dispatch path correctly reuses the existing descriptor and
  governance gates.
- Confirm transparent `SELECT ... WHERE pk = ...` remains appropriately open
  for a planner/view-hook integration packet.

## Artifacts

- `review/30840-spire-coordinator-pk-select-forwarding/artifacts/manifest.md`
- `review/30840-spire-coordinator-pk-select-forwarding/artifacts/cargo-test-forward-coordinator-select-lib.log`
- `review/30840-spire-coordinator-pk-select-forwarding/artifacts/cargo-fmt-check.log`
- `review/30840-spire-coordinator-pk-select-forwarding/artifacts/git-diff-check.log`
