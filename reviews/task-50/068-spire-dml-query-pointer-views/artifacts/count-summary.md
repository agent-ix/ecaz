# Count Summary

- code commit: `ffa04340af2d0b9aef2a4b3a38172fdb84643630`
- packet: `reviews/task-50/068-spire-dml-query-pointer-views/`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 37 | 34 | -3 |
| `src/` total | 2081 | 2078 | -3 |

## Notes

- Advances Task 50 comprehensive plan program P11, planner/node/list views.
- Consolidates repeated `Query` pointer reads behind the private `dml_frontdoor_query_ref` view helper.
- No runtime benchmark was run because this changes planner-query pointer handling only and does not alter scoring, candidate ordering, payload bytes, WAL order, or hot-path allocation shape.
