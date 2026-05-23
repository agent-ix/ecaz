# Count Summary

- head SHA: `fb2d528f`
- task bucket: `reviews/task-50/063-spire-coordinator-live-index-view/`
- packet timestamp: `2026-05-20`
- slice: SPIRE coordinator snapshot live-index view rollout

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/snapshots.rs` | 35 | 16 | -19 |
| `src/` ledger rows | 2125 | 2106 | -19 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2106` direct unsafe rows under `src/`.

