# Count Summary

- head SHA: `5afb7b4efa20d92fee088da0e858621772fef814`
- previous SHA: `32d7516a`
- task bucket: `reviews/task-50/047-spire-dml-integer-decoders/`
- timestamp: `2026-05-20`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 76 | 73 | -3 |
| `src/` total | 2324 | 2321 | -3 |

## Disposition

- Removed caller-side unsafe wrappers around integer Datum/Const decoder helpers.
- `dml_frontdoor_param_datum_to_bigint` now owns its type-OID checks before using integer Datum accessors.
- `dml_frontdoor_const_bigint_value` now owns its null/type dispatch before using `FromDatum`.
- No unsafe was moved into a new helper in this packet.
