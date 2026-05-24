# Count Summary

- head SHA: `eb2d843f`
- task bucket: `reviews/task-50/065-spire-page-tuple-read-unsafe/`
- packet timestamp: `2026-05-20`
- slice: SPIRE page tuple read unsafe consolidation

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/page.rs` | 35 | 27 | -8 |
| `src/` ledger rows | 2099 | 2091 | -8 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2091` direct unsafe rows under `src/`.

