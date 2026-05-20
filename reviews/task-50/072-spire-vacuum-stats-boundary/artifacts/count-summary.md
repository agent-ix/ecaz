# Count Summary

- code commit: `df8d7da608c78d903d1e02a9277953ed233b0307`
- packet: `reviews/task-50/072-spire-vacuum-stats-boundary/`
- touched file: `src/am/ec_spire/vacuum/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 26 | 24 | -2 |
| `src/` total | 2069 | 2067 | -2 |

## Notes

- Advances Task 50 comprehensive plan program P1/P2 for the SPIRE vacuum callback boundary.
- Consolidates PostgreSQL vacuum stats allocation, relation block-count read, and stats mutation into one explicit `finish_vacuum_stats` boundary.
- No benchmark was run because this changes unsafe block ownership only and does not alter scoring, candidate ordering, payload bytes, WAL order, or hot-path allocation shape.
