# Count Summary

- head SHA: `10451b74`
- task bucket: `reviews/task-50/056-spire-dml-frontdoor-hook-state/`
- packet timestamp: `2026-05-20`
- slice: SPIRE DML front-door hook state and catalog boundary cleanup

## Direct Unsafe Count Delta

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 73 | 63 | -10 |
| `src/` ledger rows | 2215 | 2205 | -10 |

## Current Closeout Status

Task 50 is not complete. The packet-local ledger generated after this slice
contains `2205` direct unsafe rows under `src/`.

