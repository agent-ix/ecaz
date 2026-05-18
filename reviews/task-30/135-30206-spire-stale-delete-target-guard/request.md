---
id: 30206
title: SPIRE Stale Delete Target Guard
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 93494ea9
---

# Review Request: SPIRE Stale Delete Target Guard

## Summary

This checkpoint adds update-path regression coverage for stale heap-locator
semantics: a delete delta cannot target a row that is marked as a stale
locator, because stale rows are no longer visible delete targets.

- Constructs a published snapshot with a primary assignment carrying
  `SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR`.
- Verifies `build_delta_epoch_draft_from_snapshot` rejects a delete for that
  `vec_id`.
- Asserts PID allocator, local vec-id allocator, and object-store page count do
  not advance on the rejected draft.

## Non-Goals

- No HOT-chain repair implementation.
- No vacuum cleanup implementation.
- No heap visibility recheck.
- No AM callback wiring.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 104 selected tests passed
  - 16 `ec_spire::assign` unit tests
  - 8 `ec_spire::build` unit tests
  - 33 `ec_spire::meta` unit tests
  - 4 `ec_spire::scan` unit tests
  - 25 `ec_spire::storage` unit tests
  - 18 `ec_spire::update` unit tests
  - 2 pg catalog tests

`cargo fmt` and `cargo fmt --check` still emit the repository's existing stable
rustfmt warnings for nightly-only `imports_granularity` and `group_imports`
settings.
