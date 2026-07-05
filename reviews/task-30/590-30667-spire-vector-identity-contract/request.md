# Review Request: SPIRE Vector Identity Contract

This closes Phase 9.6 by making remote merge safe for node-local vec IDs while
preserving global vec ID dedupe for cross-node replicas.

Code checkpoint: `942d61f8` (`Define SPIRE remote vector identity contract`)

## Scope

- Adds ADR-055 for the SPIRE vector identity contract.
- Defines global `SpireVecId` bytes (`0x02 || stable_global_payload_bytes`) as
  the only cross-node dedupe identity.
- Keeps existing local `SpireVecId` bytes (`0x01 || little_endian_u64`) as a
  compatibility format, scoped by origin `node_id` during remote merge.
- Validates remote candidate `vec_id` bytes before batch merge.
- Applies the same global-or-node-scoped dedupe key to both remote candidate
  batch merge and coordinator heap-result merge.
- Adds `ec_spire_remote_search_vector_identity_contract()` and updates the
  remote merge summary dedupe key text.
- Marks Phase 9.6 complete in the Phase 9 task file and main Task 30 overview.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 remote_candidate --lib`
- `cargo test --no-default-features --features pg18 remote_heap_candidate --lib`
- `cargo test --no-default-features --features pg18 remote_local_heap_locator --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_remote_search_final_contract --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_remote_phase7_policy_contracts --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_remote_search_receive_merge_summary --lib`

## Review Focus

- Confirm node-scoped local vec IDs prevent false cross-node dedupe without
  breaking existing local-only indexes.
- Confirm global vec IDs still dedupe across nodes and preserve the existing
  score/assignment-role tie-break order.
- Check that candidate batch merge and coordinator heap-result merge use the
  same dedupe key semantics.
- Check that the SQL-visible contract and task checkboxes do not overclaim:
  writers still need a future source/global ID allocation path before
  cross-node replicas can dedupe as one vector.
