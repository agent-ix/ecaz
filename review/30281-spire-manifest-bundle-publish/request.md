# SPIRE Manifest Bundle Publish Checkpoint

## Checkpoint

- Code commit: `10f74a56` (`Publish SPIRE manifest bundle tuples`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: relation-backed manifest bundle persistence plus reviewer hardening before populated `ambuild`

## Summary

This checkpoint writes encoded SPIRE epoch/object/placement manifest bundles as relation object tuples and publishes root/control state to those manifest locators for an empty epoch. It keeps populated `ambuild` out of scope; the pg_test path creates an empty `ec_spire` index, writes the manifest bundle through the relation tuple path, flips root/control to `active_epoch = 1`, and reads the persisted root/control state back.

The slice also folds in the high-priority live-persistence feedback from the local second architecture pass:

- F1: publish coordinator object/placement stage transitions now consume write evidence and reject missing or mismatched evidence before manifest encoding/validation.
- F3: relation object tuple append now consults the FSM before allocating a new data block.
- F4: relation object tuple append now rejects writes before the root/control block exists, including the new-block path.
- F5: root/control reads now bounds-check the special area before decoding.
- F8: `SpireRelationObjectStore` documents why mutation uses `&self` under PostgreSQL buffer/WAL locking.

Deferred from that feedback for later populated-build work:

- F2: pinned object tuple reader for zero-copy page-borrowed decode.
- F6: scan opaque root/control caching across rescans.
- F7: populated `ambuild` end-to-end publish coordinator driving order.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/storage.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib publish_coordinator --no-default-features --features pg18`
  - `3 passed; 0 failed; 0 ignored; 0 measured; 1066 filtered out`
- `cargo test --lib test_ec_spire_empty_manifest_publish_roundtrip --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1068 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `188 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- The local untracked reviewer feedback files under `review/30219-spire-foundation-progress-status/feedback.md` and `review/30255-spire-foundation-architecture-response/feedback.md` were read for context where relevant but intentionally not staged in this checkpoint.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
