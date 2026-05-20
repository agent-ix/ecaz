# Task 50 Packet 109 Count Summary

- Head SHA: `00a042fb9b15db43390173de3190d48d55f18153`
- Scope: SPIRE remote write wrapper boundary cleanup
- Previous packet total: `1706` direct unsafe blocks under `src/`
- Current total: `1694` direct unsafe blocks under `src/`
- Delta: `-12`

## Per-File Movement

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` | 2 | 2 | 0 |
| `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs` | 9 | 0 | -9 |
| `src/lib.rs` | 37 | 37 | 0 |
| `src/tests/insert.rs` | 16 | 15 | -1 |
| `src/tests/mod.rs` | 40 | 38 | -2 |

## Ledger Movement

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` direct unsafe blocks | 1706 | 1694 | -12 |
| `src/` unsafe ledger rows | 1706 | 1694 | -12 |
