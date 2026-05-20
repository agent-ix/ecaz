# Count Summary

- head SHA: `2b759d6c`
- task bucket: `reviews/task-50/055-spire-hierarchy-live-relation-view/`
- packet timestamp: `2026-05-20`
- slice: SPIRE coordinator hierarchy snapshot live relation view rollout

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 48 | 38 | -10 |
| `src/` ledger rows | 2225 | 2215 | -10 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2215` direct unsafe rows under `src/`.

