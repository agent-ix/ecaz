# Count Summary

- head SHA: `694a4bf5`
- task bucket: `reviews/task-50/058-spire-relation-object-reader-safe/`
- packet timestamp: `2026-05-20`
- slice: SPIRE relation object reader safe facade rollout

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/storage/relation_store.rs` | 38 | 16 | -22 |
| `src/am/ec_spire/coordinator/debug.rs` | 38 | 36 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 37 | 35 | -2 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 38 | 37 | -1 |
| `src/` ledger rows | 2200 | 2173 | -27 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2173` direct unsafe rows under `src/`.

