# Review Request: SPIRE remote node descriptor contract

## Summary

Code checkpoint: `0a105c34` (`Expose SPIRE remote node descriptor contract`)

This slice adds a SQL-visible contract for the future durable remote-node descriptor before any libpq connection-opening code lands.

- Adds `ec_spire_remote_node_descriptor_contract()`.
- Lists the descriptor fields from the Phase 7 remote-node model: coordinator index identity, node ID, generation, indirect conninfo secret, remote index identity/locator, node state, health timestamp, served/retained epoch window, extension version, and last error.
- Keeps raw connection strings out of the contract; the only connection field is `conninfo_secret_name` with the semantic role `indirect_connection_secret`.
- Updates the Phase 7 task note to mention the descriptor contract surface.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - 8 passed; 0 failed; 1426 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and validation claims.
