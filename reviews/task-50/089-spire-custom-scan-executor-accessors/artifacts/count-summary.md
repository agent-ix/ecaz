# Task 50 Packet 089 Count Summary

Head SHA: `4ff977b2333be17474eceb822ad953f011de0a56`

Program coverage:

- P10 scan opaque / raw ownership contracts
- P11 planner, node, list, and custom scan views
- Wave 2 SPIRE CustomScan executor fanout

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1955 | 1953 | -2 |
| `src/am/ec_spire/custom_scan/begin_exec.rs` | 13 | 10 | -3 |
| `src/am/ec_spire/custom_scan/dml.rs` | 14 | 15 | +1 |
| `src/am/ec_spire/custom_scan/tuple_payload.rs` | 6 | 6 | 0 |
| `src/` unsafe ledger rows | 1955 | 1953 | -2 |

Notes:

- `begin_exec.rs` caller unsafe was removed from BeginCustomScan, ReScanCustomScan, CustomScan access, and DML access dispatch paths.
- The `dml.rs` count increases by one because the production result-stream boundary is now inside a safe `custom_scan_ensure_outputs` helper. This moves caller unsafe into the owned helper boundary rather than requiring executor callbacks to call an unsafe function.
- `tuple_payload.rs` direct block count is unchanged, but tuple-payload writers now accept safe executor-state references instead of raw `SpireCustomScanExecState` pointers.

Task 50 is not complete. The regenerated ledger still contains `1953` current `src/` unsafe rows that must be removed or residual-registered.
