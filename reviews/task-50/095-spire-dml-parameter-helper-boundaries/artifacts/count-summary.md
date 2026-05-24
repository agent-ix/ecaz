# Task 50 Packet 095 Count Summary

Head SHA: `92918fd8d523947d7bf2a947be8e0d3d59986255`

Program coverage:

- P2 PostgreSQL handle views
- P5 heap source, tuple slot, snapshot, and scorer contracts
- P6 Datum, varlena, vector, and type conversion contracts
- Wave 1 SPIRE production DML front-door

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1917 | 1915 | -2 |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 30 | 28 | -2 |
| `src/tests/dml_frontdoor.rs` | 5 | 5 | 0 |
| `src/` unsafe ledger rows | 1917 | 1915 | -2 |

Notes:

- Made `dml_frontdoor_primitive_plan_pk_value_bytes` safe to call. It now
  delegates parameter extraction to one internal `ParamListInfo` boundary.
- Made `dml_frontdoor_primitive_invocation_from_plan` safe to call. The helper
  now consumes the safe PK-byte helper rather than exposing an unsafe API.
- Updated the pg_test call site that no longer needs the test macro's unsafe
  wrapper, avoiding a new `unused_unsafe` warning.

Task 50 is not complete. The regenerated ledger still contains `1915` current
`src/` unsafe rows that must be removed or residual-registered.
