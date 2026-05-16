# 30765 - SPIRE Row Materialization Mapping Contract

## Summary

This packet reviews commit `e415b20eacd71bd15918b3c13424f9a064fccb91`
(`Expose SPIRE row materialization mapping contract`).

The slice follows ADR-064 by adding the explicit contract for mapping
remote-origin, heap-resolved outputs to coordinator-visible heap TIDs before
the index AM may return them through `xs_heaptid`.

The new SQL surface is
`ec_spire_remote_search_row_materialization_mapping_contract()`. It pins:

- exact mapping identity: requested epoch, served epoch, origin node, global
  vec-id, and opaque row locator must match the heap-resolved remote output;
- same-relation AM delivery: the materialized TID must belong to the heap
  relation being scanned by the index scan;
- scan snapshot visibility: stale or vacuumed materialized rows remain blocked;
- no scan-time heap writes from `amrescan` or `amgettuple`;
- strict/degraded behavior for missing or stale mappings.

This is still a provider boundary, not remote-origin AM delivery. The scan path
continues to block remote-origin outputs with `remote_row_materialization` until
a real epoch-scoped materialized-row provider exists.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/lib.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `git diff --check -- <changed code/docs>`
- `cargo fmt --check`
- `cargo test row_materialization_mapping_contract --no-default-features --features pg18`
- `cargo test row_materialization_contract --no-default-features --features pg18`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo pgrx test pg18 test_ec_spire_remote_search_final_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`

No PostgreSQL distributed fixture or performance run was started for this
packet.

## Review Focus

- Is the mapping identity complete enough to reject stale or mismatched
  materialized rows?
- Is the same-relation TID rule explicit enough for PostgreSQL index AM
  correctness?
- Does this preserve the distinction between the new provider boundary and
  actual remote-origin AM delivery?
