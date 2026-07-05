# Review Request: SPIRE Storage Module Split

Head SHA: `78850a5e`

## Summary

This is a split-only follow-up to the SPIRE update/scan chunking work. It keeps
the same include-based namespace shape and moves the oversized storage module
into smaller files.

The old large file is now a thin include facade:

- `src/am/ec_spire/storage.rs`

The moved bodies now live under:

- `src/am/ec_spire/storage/{vec_id,relation_plan,header,assignment,leaf_v1,leaf_v2_parts,leaf_v2,routing_delta,local_store,local_store_set,relation_store,helpers,tests}.rs`
- `src/am/ec_spire/storage/tests/{vec_and_routing,local_store_plan,assignment,leaf,delta,local_store}.rs`

After this checkpoint, the largest SPIRE storage split file is under 600 lines.
The largest files are:

- `src/am/ec_spire/storage/relation_store.rs` at 587 lines
- `src/am/ec_spire/storage/local_store.rs` at 507 lines
- `src/am/ec_spire/storage/routing_delta.rs` at 460 lines
- `src/am/ec_spire/storage/leaf_v2_parts.rs` at 452 lines

No function bodies, visibility, APIs, tests, or call paths were intentionally
changed.

## Mechanical Proof

Reconstructed `src/am/ec_spire/storage.rs` by concatenating the facade header,
the new chunks, and the nested test chunks. Compared it against `HEAD^` with
blank-line-only differences ignored:

- `src/am/ec_spire/storage.rs`: matched

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo test --all-targets --no-default-features --features pg18 -- --test-threads=1`

The full serial PG18 test run passed:

- `1325 passed; 0 failed; 4 ignored` for the main library test target
- Remaining bin, integration, and bench test targets passed

Clippy was run but is not clean:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Failed on existing project-wide warnings-as-errors, including
  `unnecessary_cast`, `too_many_arguments`, `clone_on_copy`, and
  `type_complexity`; the storage split also relocates an existing test lint to
  `src/am/ec_spire/storage/tests/vec_and_routing.rs`.

## Review Focus

- Confirm this is only a mechanical split.
- Confirm the chunk names are useful enough for Phase 4 review.
- Flag any storage chunk that should be renamed before logic lands on top.
