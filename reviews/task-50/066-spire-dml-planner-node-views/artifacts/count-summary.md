# Count Summary

- code commit: `21773a706c72c5cab1a83ea14c5fb24c360fcd15`
- packet: `reviews/task-50/066-spire-dml-planner-node-views/`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 47 | 39 | -8 |
| `src/` total | 2091 | 2083 | -8 |

## Notes

- Advances Task 50 comprehensive plan program P11, planner/node/list/custom scan views.
- Consolidates repeated caller-side unsafe for DML frontdoor planner `Query`, `FromExpr`, `Node`, and `List` reads into private typed view helpers.
- No runtime benchmark was run because this changes planner-tree unsafe ownership only and does not change scoring, candidate ordering, payload bytes, WAL order, or allocation shape on a hot path.
