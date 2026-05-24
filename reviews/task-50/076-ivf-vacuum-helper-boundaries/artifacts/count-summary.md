# Count Summary

- code commit: `6c48e2842816265e6ba50b035283255e71e4ae18`
- task bucket: `reviews/task-50/076-ivf-vacuum-helper-boundaries/`
- touched production file: `src/am/ec_ivf/vacuum.rs`
- program coverage: P1 callback boundary, P2 PostgreSQL handle helpers, P3 IVF page/rewrite orchestration
- timestamp: `2026-05-20T13:07:18-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/vacuum.rs` | 23 | 18 | -5 |
| `src/` total | 2053 | 2048 | -5 |

## Notes

- Makes IVF vacuum stats, posting-list rewrite orchestration, and heap-dead callback adapter safe to call from module-private vacuum flow.
- Leaves the actual PostgreSQL allocation, block-counting, callback invocation, and page rewrite unsafe blocks inside their owner helpers.
- Keeps callback behavior, metadata accounting, and posting rewrite decisions unchanged.
