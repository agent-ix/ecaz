# Count Summary

- code commit: `05d82d5a6d80d6939c27c7bbf94eed30cdc26679`
- packet: `reviews/task-50/069-spire-dml-integer-datum-views/`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 34 | 32 | -2 |
| `src/` total | 2078 | 2076 | -2 |

## Notes

- Advances Task 50 comprehensive plan programs P6 and P11.
- Consolidates by-value integer Datum decoding for DML constants and bound parameters into one helper.
- Replaces manual one-element PG18 ListCell access for coerced expression arguments with the existing DML `PgList` view helper.
- No runtime benchmark was run because this changes planner/value helper ownership only and does not alter scoring, candidate ordering, payload bytes, WAL order, or hot-path allocation shape.
