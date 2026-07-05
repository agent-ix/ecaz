# Review Request: SPIRE Local Store Descriptor Publish Plan

- Branch: `task30-spire-partition-object-spec`
- Code commit: `9103578295f1e771a3ede028208de2a6bbfb2aa7`
- Scope: Phase 4 active-store config planning before auxiliary store DDL

## Summary

This checkpoint adds the pure planning bridge from future created auxiliary
store relation OIDs to the active local-store config metadata.

It:

- adds `local_store_config_from_relation_plan`;
- combines a sorted relation/tablespace plan with created `(local_store_id,
  store_relid)` pairs;
- produces a validated `SpireLocalStoreConfig` for the requested generation;
- preserves repeated tablespace OIDs for same-device baseline runs;
- rejects missing, duplicate, and unexpected created store relids before a
  future DDL helper can publish an active store generation.

This still does not create store relations, open relation handles, write
root/control config metadata, or remove the current `local_store_count > 1`
ambuild guard.

## Files

- `src/am/ec_spire/storage.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

- Whether `local_store_config_from_relation_plan` is the right boundary between
  catalog DDL and root/control metadata publish.
- Whether the helper should reject unexpected created relids instead of ignoring
  them. The current behavior is strict to avoid publishing a store generation
  from a mismatched DDL result set.
- Whether the generation value should stay caller-supplied at this layer.

## Validation

- `cargo test local_store_relation_plan --lib`
- `cargo test local_store_config --lib`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`
