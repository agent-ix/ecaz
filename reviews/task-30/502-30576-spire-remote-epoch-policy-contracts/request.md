# Review Request: SPIRE remote epoch policy contracts

## Summary

Code checkpoint: `e1f492a9` (`Expose SPIRE remote epoch policy contracts`)

This batch adds three related Phase 7 control-plane surfaces before real libpq
execution lands.

- Adds `ec_spire_remote_epoch_publish_plan(...)` as the per-remote-node view of
  distributed epoch publish readiness, including placement-state counts,
  required served/retained epoch windows, observed node windows, and the
  precise publish blocker.
- Adds `ec_spire_remote_degradation_policy_contract()` to expose strict vs.
  degraded placement-state actions shared by search fanout and epoch
  publication.
- Adds `ec_spire_remote_search_merge_order_contract()` to expose the comparator
  order used by the remote candidate merge helper.
- Updates the Phase 7 task note for distributed epoch publication, graceful
  degradation, and merge semantics.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 51 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
validation claims.
