# Count Summary

- code commit: `d42efe27e9ed7b6b5f499e9d3f68adee07a48d66`
- packet: `reviews/task-50/070-spire-dml-hook-diagnostics-state/`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Direct Unsafe Blocks

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 32 | 30 | -2 |
| `src/` total | 2076 | 2074 | -2 |

## Notes

- Advances Task 50 comprehensive plan program P1, callback/hook boundary contracts.
- Moves backend-local DML frontdoor hook diagnostics from `static mut` fields into a safe `Mutex<DmlFrontdoorBackendHookState>` snapshot.
- Keeps the actual PostgreSQL hook install and relcache callback registration as explicit unsafe FFI boundaries.
- No runtime benchmark was run because this changes hook diagnostic storage only and does not alter scoring, candidate ordering, payload bytes, WAL order, or hot-path allocation shape.
