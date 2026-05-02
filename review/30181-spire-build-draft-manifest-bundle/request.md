---
id: 30181
title: SPIRE Build Draft Manifest Bundle
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: b8efe336
---

# Review Request: SPIRE Build Draft Manifest Bundle

## Summary

This checkpoint adds an encoded manifest bundle for a validated SPIRE
single-level build draft. It prepares concrete persistence payloads without
writing them to PostgreSQL relations.

- Adds `SpireEncodedManifestBundle`.
- Adds `SpireSingleLevelBuildDraft::encode_manifest_bundle`.
- Revalidates the draft through `SpirePublishedEpochSnapshot`.
- Encodes:
  - epoch manifest
  - object manifest
  - placement directory
- Round-trips encoded bytes through existing metadata decoders in tests.

## Non-Goals

- No relation-backed manifest persistence.
- No root/control publish transaction.
- No AM callback behavior change.
- No object cleanup or retention execution.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 72 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 7 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
