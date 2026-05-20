# Count Summary

- head SHA: `70a8ff9f`
- task bucket: `reviews/task-50/062-spire-hierarchy-live-index-view/`
- packet timestamp: `2026-05-20`
- slice: SPIRE hierarchy snapshot live-index view rollout

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 37 | 21 | -16 |
| `src/` ledger rows | 2141 | 2125 | -16 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2125` direct unsafe rows under `src/`.

