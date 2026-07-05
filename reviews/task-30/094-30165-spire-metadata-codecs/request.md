---
id: 30165
title: SPIRE Metadata Codecs
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: b604b59a
---
# Review Request: SPIRE Metadata Codecs

## Summary

This checkpoint adds fixed V1 codecs for SPIRE root/control metadata rows. It
does not write those rows to PostgreSQL storage yet.

The checkpoint:

- replaces the `ec_spire` metadata placeholder with codecs for placement
  entries, epoch manifests, and manifest entries
- models Phase 1 local placement as `node_id = 0` and `local_store_id = 0`
  through `SpirePlacementEntry::local_single_store`
- reserves the future placement shape by storing `pid`, `node_id`,
  `local_store_id`, `store_relid`, object version, object tuple locator,
  object byte length, and placement state
- models epoch states `building`, `published`, `retired`, and `failed`
- models consistency modes `strict` and `degraded`, with strict available as
  the local single-store default for later wiring
- records Phase 0 retention defaults in code constants: 10 minute minimum
  retention, 60 minute failed-epoch retention, and active plus two retained
  published/retired epochs
- validates non-zero epoch/PID/object versions, non-zero store relids, valid
  locators, known states/modes, reserved bytes, and basic publish timestamp
  invariants
- adds focused unit tests for placement, epoch, manifest-entry, retention
  default, and rejection cases

No active-epoch publication, failed-publish cleanup, relation page layout,
build path, scan path, insert path, or vacuum repair behavior is included in
this checkpoint.

## Files To Review

- `src/am/ec_spire/meta.rs`

## Design Alignment

This follows the Phase 0 decision record in
`plan/design/spire-phase0-partition-object-storage.md`:

- placement remains `pid -> local_store_id -> object` for local Phase 1
- the row format keeps `node_id` so later remote placement can extend to
  `pid -> node_id -> local_store_id -> object`
- manifests reference per-partition object versions instead of duplicating
  full `(pid, epoch)` objects
- failed/building manifests are represented as states but are not active until
  a later root/control publish operation advances `active_epoch`
- retention defaults are durable constants rather than implicit behavior

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo test --lib ec_spire --no-default-features --features pg18`

`cargo fmt` and `cargo fmt --check` emit the repository's existing
stable-toolchain warnings for unstable rustfmt options
(`imports_granularity`, `group_imports`), but formatting passed.

The focused test command passed 21 selected tests:

- 11 `ec_spire::meta` unit tests
- 8 `ec_spire::storage` unit tests
- 2 existing `ec_spire` pg catalog registration tests

## Reviewer Focus

1. Is the placement-entry byte shape sufficient for Phase 1 while preserving
   the future `pid -> node_id -> local_store_id` extension?
2. Should unavailable or skipped placements require an object locator now, or
   should those states be allowed to carry no object location before degraded
   mode is wired?
3. Are the epoch manifest state and timestamp invariants too strict, too weak,
   or right for the first publish implementation?
4. Are the retention defaults represented at the right layer before cleanup
   code exists?
