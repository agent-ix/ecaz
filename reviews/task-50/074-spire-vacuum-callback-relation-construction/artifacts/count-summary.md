# Count Summary

- code commit: `70de562aab4c1452d3ff25869bbbd4b7ca70a013`
- task bucket: `reviews/task-50/074-spire-vacuum-callback-relation-construction/`
- touched production file: `src/am/ec_spire/vacuum/mod.rs`
- program coverage: P1 callback boundary, P2 PostgreSQL relation views
- timestamp: `2026-05-20T13:01:36-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 20 | 16 | -4 |
| `src/` total | 2063 | 2059 | -4 |

## Notes

- Replaces caller-side unsafe construction of the private SPIRE vacuum relation view with a module-private safe constructor used only from vacuum callback paths.
- Makes the heap-dead callback adapter safe to call from bulk-delete orchestration; the actual PostgreSQL callback invocation remains inside the helper.
- Leaves remaining direct unsafe in named PostgreSQL boundary wrappers, relation/page helper methods, stats allocation/mutation, and test/debug callback paths.
