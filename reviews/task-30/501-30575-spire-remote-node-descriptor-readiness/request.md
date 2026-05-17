# Review Request: SPIRE remote node descriptor readiness

## Summary

Code checkpoint: `d839212c` (`Expose SPIRE remote descriptor readiness`)

This slice projects the remote-node descriptor contract onto discovered remote
nodes so the pre-libpq blocker is visible per required field.

- Adds `ec_spire_remote_node_descriptor_readiness(...)`.
- Adds `ec_spire_remote_node_descriptor_readiness_summary(...)`.
- Reports missing required descriptor fields as `missing_descriptor`, while
  optional missing fields remain non-blocking.
- Keeps raw connection strings out of the readiness output; the contract still
  exposes only the indirect `conninfo_secret_name` field.
- Updates the Phase 7 task note to mention the readiness surfaces.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - 9 passed; 0 failed; 1426 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
validation claims.
