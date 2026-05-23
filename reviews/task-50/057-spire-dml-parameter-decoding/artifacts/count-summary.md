# Count Summary

- head SHA: `7a0eb77f`
- task bucket: `reviews/task-50/057-spire-dml-parameter-decoding/`
- packet timestamp: `2026-05-20`
- slice: SPIRE DML front-door executor parameter decoding cleanup

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 63 | 58 | -5 |
| `src/` ledger rows | 2205 | 2200 | -5 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2200` direct unsafe rows under `src/`.

