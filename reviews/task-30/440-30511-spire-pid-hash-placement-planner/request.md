# Review Request: SPIRE PID Hash Placement Planner

## Checkpoint

- Code commit: `5826eb7d`
  (`Add SPIRE PID hash placement planner`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 deterministic hash placement planning

## Summary

This checkpoint adds the deterministic local-store placement primitive that the
future multi-store writer path will use.

`src/am/ec_spire/meta.rs` now has:

- `spire_pid_hash(pid)`, a fixed SplitMix64-style finalizer owned by SPIRE;
- `SpireLocalStoreConfig::store_for_pid(pid)`, which selects
  `hash(pid) % store_count` over the active store descriptor list;
- focused tests for stable hash outputs and hash-mod-store selection;
- an explicit rejection for invalid PID 0 placement.

The Task 30 tracker now splits the Phase 4 hash work into a completed planning
primitive and an open hash-routed object-writes item.

## Files

- `src/am/ec_spire/meta.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether the fixed hash function is acceptable as SPIRE's durable placement
  rule;
- whether stable expected hash values are enough to guard cross-platform drift;
- whether `store_for_pid` should return a store descriptor rather than only the
  store ID;
- whether root/routing/leaf object writers can safely consume this helper in
  the next slice.

## Validation

- `cargo fmt --check`
- `cargo test spire_pid_hash --lib`
- `cargo test local_store_config_places_pid --lib`
- `git diff --check`
- `git diff --cached --check`

## Notes

No PostgreSQL or PG18 tests were run. This slice does not yet route relation
writes; it only defines and tests the placement planning rule.
