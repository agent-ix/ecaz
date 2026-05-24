# Task 50 Packet 096 Count Summary

Head SHA: `128d3324024c6e8f052cb99f8e975b2b7f805560`

Program coverage:

- P2 PostgreSQL handle views
- P3 buffer, page, and WAL transaction contracts
- P4 page tuple and line-pointer views
- Wave 1 SPIRE production page/build/publish/scan/update/vacuum fanout

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1915 | 1826 | -89 |
| `src/am/ec_spire/build/drafts.rs` | 17 | 14 | -3 |
| `src/am/ec_spire/build/publish.rs` | 9 | 0 | -9 |
| `src/am/ec_spire/build/recursive.rs` | 3 | 0 | -3 |
| `src/am/ec_spire/coordinator/debug.rs` | 29 | 9 | -20 |
| `src/am/ec_spire/coordinator/diagnostics.rs` | 9 | 4 | -5 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 21 | 18 | -3 |
| `src/am/ec_spire/coordinator/maintenance.rs` | 19 | 16 | -3 |
| `src/am/ec_spire/coordinator/remote_candidates/endpoint_identity.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` | 12 | 10 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/fault_matrix.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 24 | 22 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 16 | 11 | -5 |
| `src/am/ec_spire/custom_scan/planner.rs` | 12 | 10 | -2 |
| `src/am/ec_spire/insert.rs` | 18 | 12 | -6 |
| `src/am/ec_spire/page.rs` | 27 | 19 | -8 |
| `src/am/ec_spire/scan/relation.rs` | 15 | 11 | -4 |
| `src/am/ec_spire/scan/types.rs` | 1 | 0 | -1 |
| `src/am/ec_spire/storage/relation_plan.rs` | 14 | 13 | -1 |
| `src/am/ec_spire/storage/relation_store.rs` | 16 | 15 | -1 |
| `src/am/ec_spire/update/publish/relation.rs` | 9 | 3 | -6 |
| `src/am/ec_spire/vacuum/mod.rs` | 15 | 12 | -3 |
| `src/` unsafe ledger rows | 1915 | 1826 | -89 |

Notes:

- Made SPIRE page root-control read/init, object tuple append/read, object
  tuple scan, rewrite, delete, and publish-manifest helpers safe to call.
- Kept raw buffer/page/WAL access in `src/am/ec_spire/page.rs`; broad callers
  now use safe page/publish helper contracts.
- `src/am/ec_spire/build/publish.rs`, `src/am/ec_spire/build/recursive.rs`,
  and `src/am/ec_spire/scan/types.rs` now have zero direct unsafe blocks.

Task 50 is not complete. The regenerated ledger still contains `1826` current
`src/` unsafe rows that must be removed or residual-registered.
