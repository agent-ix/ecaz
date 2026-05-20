# Count Summary

- code commit: `9edc102f05bd3bdc3e50ee347a89327602b8a999`
- task bucket: `reviews/task-50/073-spire-vacuum-publish-relation-view/`
- touched production file: `src/am/ec_spire/vacuum/mod.rs`
- program coverage: P1 callback boundary, P2 PostgreSQL relation views, P3 SPIRE publish/page contract
- timestamp: `2026-05-20T12:58:23-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 24 | 20 | -4 |
| `src/` total | 2067 | 2063 | -4 |

## Notes

- Routes the SPIRE vacuum publish lock and replacement-epoch publish helpers through `SpireVacuumIndexRelation`.
- Removes caller-side relation pointer unsafe from compacted-delete and delete-delta publish paths.
- Keeps the remaining vacuum unsafe concentrated in callback entry, relation-view construction, stats/debug boundaries, and callback invocation surfaces for later residual disposition or further helper extraction.
