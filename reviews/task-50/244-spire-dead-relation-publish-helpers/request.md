# Task 50 Review Request: SPIRE Dead Relation Publish Helpers

## Summary

Removed unused SPIRE relation-oriented helper surfaces:

- `build_relation_recursive_routing_epoch_draft`
- `build_relation_recursive_top_graph_epoch_draft`
- `build_relation_recursive_routing_epoch_from_leaf_inputs`
- `publish_relation_replacement_epoch_from_object_placements`
- orphaned `SpireRelationReplacementEpochObjectPlacementInput`

The remaining local recursive build helpers and scheduled relation replacement
publish path are unchanged. `rustfmt` also mechanically wrapped three existing
long expressions in `src/am/ec_spire/build/recursive.rs` after the touched-file
format check exposed drift.

## Unsafe Burndown

- touched SPIRE file unsafe grep count: `10 -> 5`
- repository `src` unsafe grep count: `2478 -> 2473`
- deleted-symbol search returns no remaining references

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/build/recursive.rs src/am/ec_spire/update/publish/relation.rs src/am/ec_spire/update/types.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify these removed relation build/publish helpers were genuinely dead
and that the remaining scheduled relation replacement publish path still covers
the live production path.
