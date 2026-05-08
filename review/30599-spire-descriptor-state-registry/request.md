# SPIRE Descriptor State Registry

## Summary

This checkpoint closes the descriptor lifecycle follow-up from the reviewer
feedback by exposing all descriptor states through one Rust-backed registry.

Changes:

- Adds `ec_spire_remote_node_descriptor_state_contract()`.
- Registers catalog states `active`, `draining`, `disabled`, and `failed`.
- Registers synthetic state `missing` separately from catalog-writable states.
- Exposes read eligibility, snapshot status, and recommendation per state.
- Routes `ec_spire_register_remote_node_descriptor(...)` validation through the
  catalog-state registry instead of a local string-literal match.
- Updates the Phase 7 task note with the descriptor state contract.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `5a0ebe01`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor_state_contract`
- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check`

Result:

- PG18 `remote_node_descriptor_state_contract` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_state_contract`
- The test proves the four catalog states are present, only active/draining are
  read-eligible, failed maps to `failed_remote_node`, and `missing` is synthetic.

## Notes

`sql/bootstrap.sql` still contains the SQL CHECK expression because PostgreSQL
cannot consume the Rust registry in DDL. The new contract gives reviewers and
operators one SQL-visible source to compare against that boundary.
