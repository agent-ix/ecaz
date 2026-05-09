# SPIRE Secret Key Collision Guard

## Scope

Addresses reviewer feedback from `30638`/`30642`: distinct
`conninfo_secret_name` values such as `node-1` and `node_1` sanitized to the
same external provider lookup key.

Code checkpoint: `3ab80414` (`Reject SPIRE conninfo secret key collisions`)

## Changes

- Promoted the conninfo secret provider lookup-key derivation into a shared AM
  helper, so registration and status surfaces use the exact same mapping.
- `ec_spire_register_remote_node_descriptor(...)` now locks
  `ec_spire_remote_node_descriptor` during the collision check/upsert and
  rejects a different existing `conninfo_secret_name` for the same coordinator
  index when it maps to the same provider lookup key.
- Updated descriptor and registration contracts from
  `must_be_nonempty_secret_reference` to
  `must_be_nonempty_noncolliding_secret_reference`.
- Added PG18 coverage proving `node-1` then `node_1` fails with the shared
  provider lookup key `EC_SPIRE_REMOTE_CONNINFO_NODE_1`.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_secret_key_collision_rejected`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract`
- `git diff --check`

## Review Focus

- Whether allowing the same exact `conninfo_secret_name` across multiple nodes
  is the right sharing behavior. This slice only rejects different names that
  alias to the same provider lookup key.
