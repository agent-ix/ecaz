# SPIRE Descriptor Lifecycle Strings

## Summary

This packet addresses the non-blocking descriptor lifecycle feedback from
30571/30575 by moving the remaining descriptor lifecycle strings into the
shared remote constants block:

- `SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE`
- `SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING`
- `SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR`
- `SPIRE_REMOTE_STATUS_OPTIONAL_DESCRIPTOR_MISSING`

`snapshots.rs` now uses these constants for descriptor readiness, descriptor
summary counts, capability rows, epoch publish readiness, and empty-node
snapshot rows. SQL-visible string values are unchanged.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/snapshots.rs`

## Validation

Head SHA: `2dbeb51d`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor`

Result:

- PG18 descriptor filter passed 3 tests:
  - `pg_test_ec_spire_remote_node_descriptor_registration_contract`
  - `pg_test_ec_spire_remote_node_descriptor_contract`
  - `pg_test_ec_spire_remote_node_descriptor_readiness_missing`

## Notes

This is intentionally a constant-sharing slice only. The next descriptor
registration implementation can add new lifecycle states without reintroducing
ad hoc literals.
