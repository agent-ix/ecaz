# Task 50 Packet 108 Count Summary

- Head SHA: `76eb15e6c9f61b66b9ae83dbcf8480c87b590d26`
- Scope: SPIRE remote candidate wrapper boundary cleanup
- Previous packet total: `1743` direct unsafe blocks under `src/`
- Current total: `1706` direct unsafe blocks under `src/`
- Delta: `-37`

## Per-File Movement

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/lib.rs` | 37 | 37 | 0 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 18 | 17 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs` | 7 | 5 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` | 12 | 5 | -7 |
| `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` | 8 | 4 | -4 |
| `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` | 7 | 2 | -5 |
| `src/am/ec_spire/coordinator/remote_candidates/operator.rs` | 5 | 3 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs` | 5 | 4 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 16 | 2 | -14 |
| `src/am/ec_spire/custom_scan/dml.rs` | 15 | 14 | -1 |

## Ledger Movement

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` direct unsafe blocks | 1743 | 1706 | -37 |
| `src/` unsafe ledger rows | 1743 | 1706 | -37 |
