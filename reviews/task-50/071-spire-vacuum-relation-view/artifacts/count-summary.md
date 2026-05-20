# Count Summary

- code commit: `160a8fcde6cf287604cf739401c35c962b639349`
- packet: `reviews/task-50/071-spire-vacuum-relation-view/`
- touched file: `src/am/ec_spire/vacuum/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 31 | 26 | -5 |
| `src/` total | 2074 | 2069 | -5 |

## Notes

- Advances Task 50 comprehensive plan programs P1, P2, and P3 for SPIRE production vacuum/publish paths.
- Introduces a private `SpireVacuumIndexRelation` view for repeated live relation operations: root/control reads, active manifest loads, local store config loads, object-store set opens, placement writes, and replacement epoch publishing.
- Keeps PostgreSQL callback entry, publish locking, vacuum stats allocation/mutation, and heap-dead callback boundaries explicit.
- No benchmark was run because this changes unsafe ownership around vacuum relation helper calls and does not alter scoring, candidate ordering, payload bytes, WAL order, or hot-path allocation shape.
