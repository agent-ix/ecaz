---
id: 30182
title: SPIRE Encoded Publish Bundle
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 747dd3b5
---

# Review Request: SPIRE Encoded Publish Bundle

## Summary

This checkpoint adds a pure encoded publish bundle for a SPIRE single-level
build draft. It combines encoded manifest payloads with encoded root/control
state, using externally supplied manifest locators.

- Adds `SpireEncodedPublishBundle`.
- Adds `SpireSingleLevelBuildDraft::encode_publish_bundle`.
- Reuses `encode_manifest_bundle`.
- Reuses `root_control_state` to validate manifest locators.
- Encodes root/control state after the root/control validator accepts the
  supplied locators.
- Round-trips the encoded root/control state in tests.

## Non-Goals

- No PostgreSQL relation writes.
- No root/control publish transaction.
- No AM callback behavior change.
- No manifest/object cleanup execution.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 73 selected tests passed
  - 12 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 3 `ec_spire::scan` unit tests
  - 15 `ec_spire::storage` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
