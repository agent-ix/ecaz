# Task 50 Review Request: IVF Relation Scalar Handles

## Summary

This slice advances the Task 50 IVF PostgreSQL-handle view cleanup.

Code commit: `2e4bc4b1db5e5f59002beb5c4b7d416edd1ccca8`

Changed IVF page and scan relation scalar reads to use the shared
`storage::relation` handle helpers instead of local direct relation pointer
access:

- `IvfPageRelation::relid()` now uses `relation_oid_handle`.
- `IvfPageRelation::number_of_blocks()` now uses
  `main_fork_block_count_handle`.
- `ivf_index_heap_oid()` now uses `index_heap_relation_oid_handle`.

This keeps the relation scalar unsafe boundary centralized in
`src/storage/relation.rs`.

## Unsafe Counts

- `src/am/ec_ivf/page.rs`: `18 -> 16`
- `src/am/ec_ivf/scan.rs`: `24 -> 23`
- `src/` total direct unsafe blocks: `1187 -> 1184`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P2, PostgreSQL handle views.
- Wave/tranche: Wave 2, IVF/RaBitQ production fanout.
- Disposition: local relation scalar pointer reads were absorbed into the
  existing shared relation-handle boundary.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_ivf/page.rs src/am/ec_ivf/scan.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1184` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
