# SPIRE Remote Epoch Publish Gate

## Summary

This checkpoint adds the final pre-publish gate summary for Phase 7 distributed
epoch publication readiness.

Changes:

- Adds `ec_spire_remote_epoch_publish_gate_summary(index_oid)`.
- Composes `ec_spire_remote_epoch_publish_readiness(...)` into one decision row:
  local-only publish, distributed publish-ready, or blocked distributed publish.
- Reports the next publish blocker as either `remote_node_descriptor`,
  `remote_epoch_window`, `build_index`, or `none`.
- Points the gate at `ec_spire_remote_degradation_policy_contract` as the shared
  strict/degraded publish policy contract.
- Extends focused PG18 SQL coverage for local-only and missing-descriptor
  remote placement cases.
- Updates the Phase 7 task note with the new pre-publish gate surface.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `d2a6c168`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_cap_summary`
- `git diff --check`

Result:

- PG18 `remote_node_cap_summary` filter passed:
  - `pg_test_ec_spire_remote_node_cap_summary_local`
  - `pg_test_ec_spire_remote_node_cap_summary_missing`
- `cargo fmt --check` was run and still reports only the pre-existing
  unrelated rustfmt differences in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`

## Notes

This gate is still a control-plane/readiness surface. It does not publish
remote epochs or persist remote node descriptors; those remain under the Phase 7
distributed epoch and coordinator transport work.
