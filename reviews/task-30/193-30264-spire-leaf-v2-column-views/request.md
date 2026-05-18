# Review Request: SPIRE Leaf V2 Column Views

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `0f337227 Add SPIRE V2 leaf column views`

## Scope

This packet covers a partial A2 pre-persistence architecture feedback slice:
decoded V2 leaf partition objects now expose borrowed column views over their
column-major storage.

Changed files:

- `src/am/ec_spire/storage.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added `SpireLeafObjectColumns`, a borrowed view over V2 leaf segment columns:
  flags, fixed-stride vec_id bytes, heap TIDs, gammas, and encoded payloads.
- Added `SpireLeafObjectColumnRowRef` for bounds-checked row access without
  cloning the row payload.
- Added `SpireLeafPartitionObjectV2Segment::columns` and
  `SpireLeafPartitionObjectV2::column_segments` so callers can validate once
  and iterate segment column views.
- Added tests covering the V2 column metadata, first/last row access,
  local vec_id decode through the row ref, heap TID/gamma/payload borrowing,
  out-of-bounds row rejection, and empty-object column views.
- Updated the Task 30 plan and architecture feedback response to record this
  checkpoint while keeping scan migration open.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `175 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This checkpoint still borrows from the decoded in-memory V2 object. The same
view shape is intended to carry forward when the persistence reader can borrow
directly from page-backed bytes. A2 remains open until persisted scan code moves
off allocation-heavy V1 row reads and onto V2 column views plus batch scoring.
