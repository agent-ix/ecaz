# Task 50 Packet 098 Count Summary

Code commit: `f84ec0b6188bc5cd1ff383de6defe61ba8811837`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1817 | 1810 | -7 |
| `src/am/ec_spire/coordinator/debug.rs` | 8 | 7 | -1 |
| `src/am/ec_spire/coordinator/diagnostics.rs` | 2 | 0 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 20 | 18 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 10 | 9 | -1 |
| `src/am/ec_spire/storage/relation_store.rs` | 15 | 15 | 0 |
| `src/am/ec_spire/vacuum/mod.rs` | 12 | 11 | -1 |
| `src/` unsafe ledger rows | 1817 | 1810 | -7 |

## Deleted Unsafe Call Sites

- SPIRE coordinator diagnostics no longer contains direct unsafe blocks.
- Relation-backed object store set callers no longer wrap
  `for_index_relation_and_placements` in unsafe at debug, diagnostics,
  snapshot, production scan-output, and vacuum call sites.

## Residual Boundary

The remaining raw relation field read is centralized inside
`SpireRelationObjectStoreSet::for_index_relation_and_placements` after a null
check. The constructor owns relation-store mapping validation and any additional
store relation opens through `OpenedRelationsGuard`.
