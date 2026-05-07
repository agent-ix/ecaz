# Review Request: SPIRE Relation Object Tuples

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `486ccdd1 Add SPIRE relation object tuples`

## Summary

This checkpoint adds the first relation-backed object storage primitive after
the empty root/control page checkpoint. It appends encoded SPIRE object bytes to
ordinary index data blocks after metadata block 0, reads those bytes back
through buffer-cache page access, and validates the path by round-tripping an
encoded root routing object in a live `ec_spire` index relation.

## Changed Files

- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## What Changed

- Added `append_object_tuple` and `read_object_tuple` helpers for SPIRE-owned
  object bytes in relation-backed index data blocks.
- Object tuple writes skip metadata block 0, append to the last data block when
  space allows, allocate a new data block otherwise, and use the existing
  GenericXLog full-image pattern.
- Object tuple reads validate data-block addressing, offset range, item slot
  state, and tuple bounds before copying bytes out of the locked buffer.
- Added a test-only debug helper that writes an encoded SPIRE routing object,
  rereads it from the relation, decodes it, and confirms the root/control page
  remains readable with active epoch 0.
- Added pg coverage for the relation object tuple round-trip against an
  `ec_spire` index created on an empty heap.
- Updated Task 30 status to show that relation-backed object byte append/read
  exists while placement-directory persistence and populated build integration
  remain open.

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable `imports_granularity` and `group_imports` settings.
- `cargo test --lib test_ec_spire_relation_object_tuple_roundtrip --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1064 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `184 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

## Notes For Reviewer

- This is still a storage primitive, not populated `ambuild` persistence. The
  next narrow slice should wrap these raw bytes in a relation-backed SPIRE
  object store that can emit placement entries and feed snapshot loading.
- The write path currently targets the index relation's single local store
  (`local_store_id = 0`) and leaves future multi-store selection to the
  placement-map integration slice.
- The untracked architecture-review feedback file
  `review/30219-spire-foundation-progress-status/feedback.md` remains local and
  was not staged or committed by this checkpoint.
