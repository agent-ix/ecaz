---
id: 30190
title: SPIRE Delta Publish Bundle
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 1133cbbd
---

# Review Request: SPIRE Delta Publish Bundle

## Summary

This checkpoint gives in-memory SPIRE delta epoch drafts the same encoded
publish-bundle shape as single-level build drafts.

- Adds `SpireDeltaEpochDraft::encode_manifest_bundle`.
- Adds `SpireDeltaEpochDraft::root_control_state`.
- Adds `SpireDeltaEpochDraft::encode_publish_bundle`.
- Reuses `SpireEncodedManifestBundle`, `SpireEncodedPublishBundle`, and
  `SpirePublishedManifestLocators`.
- Validates the draft snapshot before encoding manifests or root-control state.
- Tests decoded manifests and root-control cursors from the encoded bundle.

## Non-Goals

- No root/control relation write.
- No PostgreSQL relation-backed object storage.
- No `aminsert` or vacuum callback wiring.
- No delta merge/compaction.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 91 selected tests passed
  - 15 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 21 `ec_spire::storage` unit tests
  - 8 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
