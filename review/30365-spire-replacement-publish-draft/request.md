# SPIRE Replacement Publish Draft

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE Phase 2 update mechanics
- Scope: Pure publish-draft helper for replacement epochs

## Summary

This checkpoint adds the pure draft assembly step after replacement object and
placement planning.

`src/am/ec_spire/update.rs` now has a replacement-epoch draft helper that:

- accepts a planned replacement placement directory
- accepts durable placement-write evidence for that directory
- builds a published epoch manifest
- derives the object manifest from placement-write evidence
- validates the epoch snapshot shape
- exposes the same manifest-bundle, root/control-state, and publish-bundle
  helpers used by existing delta and build drafts
- validates replacement leaf object inputs before future object writes, proving
  leaf-input PIDs match replacement routing children exactly and rows are
  normalized base-leaf rows without delta-insert flags or duplicate `vec_id`s

This keeps split/merge implementation aligned with the existing publish
coordinator contract before adding any live scheduler, SQL entrypoint, or
relation-backed publish path.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_epoch_draft --lib`
- `cargo test replacement_leaf_object_inputs --lib`
- `git diff --check`

## Notes

- No live scheduler, SQL entrypoint, or relation publish path is added.
- No measurement claims.
- PQ-FastScan populated support, remote placement, and replicas remain
  deferred.
