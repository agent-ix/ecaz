# Review Request: SPiRE Relation Object Write Boundary

## Summary

This slice makes relation-backed SPiRE object writes safe after object-store
construction and centralizes the raw PostgreSQL page append in one helper.

Code checkpoint: `110c0dc217da4330beded2e9c87482a6a5ec6853`

## Safety Handling

- Made these `SpireRelationObjectStore` write methods safe:
  - `insert_routing_object()`
  - `insert_leaf_object_v2_from_rows()`
  - `insert_delta_object()`
  - `insert_top_graph_object()`
- Made the corresponding `SpireRelationObjectStoreSet` dispatch methods safe,
  including `insert_delta_object_for_base_placement()`.
- Added `append_object_tuple()` as the single local unsafe boundary for
  relation-backed append writes. Its SAFETY contract is that relation object
  stores are constructed from live PostgreSQL relations owned by the caller or
  by the store set.
- Removed scattered unsafe wrappers from build, insert, vacuum, coordinator
  diagnostics, and replacement update call sites.

The relation-opening constructors remain unsafe where they receive or open raw
PostgreSQL `Relation` pointers. This slice moves the unsafe obligation to
construction and page append, instead of requiring every object-write caller to
repeat it.

## Baseline Delta

- Before: 4,782 unsafe baseline entries across 108 files.
- After: 4,748 unsafe baseline entries across 106 files.
- Net: 34 entries removed, 2 files removed from the unsafe baseline.

Changed production file counts:

- `src/am/ec_spire/build/drafts.rs`: 20 -> 18
- `src/am/ec_spire/build/object_store.rs`: 6 -> 0
- `src/am/ec_spire/coordinator/debug.rs`: 65 -> 60
- `src/am/ec_spire/insert.rs`: 24 -> 21
- `src/am/ec_spire/storage/relation_store.rs`: 69 -> 55
- `src/am/ec_spire/update/types.rs`: 2 -> 0
- `src/am/ec_spire/vacuum/mod.rs`: 41 -> 39

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `git diff --check HEAD^ HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passes with the existing PostgreSQL header warnings and existing
unused SPIRE re-export warning.

## Artifacts

- `artifacts/unsafe-baseline-before.log`
- `artifacts/unsafe-baseline-after.log`
- `artifacts/audit-unsafe.log`
- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18.log`

## Review Focus

- Is making relation object writes safe after store construction the right
  boundary, with relation validity handled by constructors/store sets?
- Is `append_object_tuple()` an acceptable single unsafe page-append boundary?
- Did this remove any caller-side safety obligation that should remain explicit?
