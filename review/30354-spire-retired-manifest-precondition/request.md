# 30354 SPIRE Retired Manifest Precondition

## Request

Review the small hardening slice for retired epoch manifest publication.

## Scope

- Factored retired-manifest construction into a pure
  `retired_epoch_manifest_from` helper.
- Added unit coverage proving an already-retired manifest is rejected before
  relation I/O.
- Updated Task 30 status to record the precondition coverage.

## Behavior

Replacement epoch publishes must retire the previous active/published manifest
only. Passing an already-retired manifest now remains a loud error at the pure
construction boundary:

`ec_spire can only retire a previously published epoch manifest`

The relation-backed write helper still appends only after the pure helper
validates and encodes the retired manifest.

## Validation

- `cargo fmt`
- `cargo test retired_epoch_manifest_requires_published_input --no-default-features --features pg18`
- `git diff --check`

