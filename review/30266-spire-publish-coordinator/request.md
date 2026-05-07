# Review Request: SPIRE Publish Coordinator

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `165cbdee Add SPIRE publish coordinator state machine`

## Scope

This packet covers the A9 pre-persistence architecture feedback slice: build and
delta publication helpers now use a typed coordinator before active epoch
publication can be encoded.

Changed files:

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added explicit publish stages:
  `WritingObjects -> WritingPlacements -> WritingManifest -> Validating ->
  PublishingActiveEpoch`.
- Added `SpirePublishFailed` carrying the failed stage and underlying error.
- Added typed state structs so root/control bytes are only produced from the
  `PublishingActiveEpoch` state.
- Centralized manifest-bundle, root/control-state, and full publish-bundle
  encoding through shared publish helper functions.
- Rewired single-level build, partitioned build, and delta draft publish
  helpers to use the coordinator instead of duplicating validation order.
- Added a regression test proving a manifest/placement validation failure stops
  before active epoch publication.
- Marked the Task 30 publish-coordinator feedback item complete and recorded
  the implementation checkpoint in the architecture feedback response.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `177 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This is still pre-persistence. The coordinator models publication order and
failed transition behavior for the in-memory manifest/root-control bundle path;
relation-backed writes will need to persist failed/building epoch manifests and
cleanup eligibility through this same state boundary.
