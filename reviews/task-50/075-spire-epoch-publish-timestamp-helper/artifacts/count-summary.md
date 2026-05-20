# Count Summary

- code commit: `7a406cdbb827be4d5a7e0c022810c34422dd4030`
- task bucket: `reviews/task-50/075-spire-epoch-publish-timestamp-helper/`
- touched production files:
  - `src/am/ec_spire/build/drafts.rs`
  - `src/am/ec_spire/coordinator/maintenance.rs`
  - `src/am/ec_spire/insert.rs`
  - `src/am/ec_spire/vacuum/mod.rs`
- program coverage: P1 callback/boundary consolidation, P3 SPIRE publish contract
- timestamp: `2026-05-20T13:04:29-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/build/drafts.rs` | 19 | 17 | -2 |
| `src/am/ec_spire/coordinator/maintenance.rs` | 20 | 19 | -1 |
| `src/am/ec_spire/insert.rs` | 20 | 18 | -2 |
| `src/am/ec_spire/vacuum/mod.rs` | 16 | 15 | -1 |
| `src/` total | 2059 | 2053 | -6 |

## Notes

- Makes `build::current_epoch_publish_times` safe to call; it still owns the single PostgreSQL `GetCurrentTimestamp` unsafe boundary internally.
- Removes caller-side timestamp unsafe from SPIRE build publish, scheduled maintenance, insert publish, and vacuum publish paths.
- Keeps timestamp overflow checks and publish metadata behavior unchanged.
