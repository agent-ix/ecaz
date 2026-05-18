# SPIRE Descriptor Registration Contract

## Summary

This packet exposes the SQL-visible contract for registering durable SPIRE
remote-node descriptors:

- `ec_spire_remote_node_descriptor_registration_contract()` returns the ordered
  validation and persistence steps that must pass before libpq fanout can use a
  remote descriptor.
- The contract keeps connection details indirect through
  `conninfo_secret_name` and `persist_secret_reference_only`; no raw conninfo
  field is introduced.
- Remote index identity and regclass resolution are explicit registration-time
  checks. The `remote_index_regclass` contract row documents that resolution
  happens against the remote node catalog, not the coordinator catalog.
- Served-epoch window, extension version, policy state, and generation
  replacement each have named validators and failure statuses.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `b4ea90a2`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor`

Result:

- PG18 descriptor filter passed 3 tests:
  - `pg_test_ec_spire_remote_node_descriptor_registration_contract`
  - `pg_test_ec_spire_remote_node_descriptor_contract`
  - `pg_test_ec_spire_remote_node_descriptor_readiness_missing`

## Notes

Reviewer feedback from 30574 asked for clarity that
`remote_index_regclass` resolves on the remote node. This packet includes that
clarifying source comment while adding the registration contract.
