# Task 50 Packet 047: SPIRE DML Integer Decoders

This packet continues the comprehensive unsafe burndown plan in SPIRE DML frontdoor value decoding.

## Change

- Made `dml_frontdoor_param_datum_to_bigint` safe. The helper checks the PostgreSQL type OID before selecting the matching by-value integer Datum accessor.
- Made `dml_frontdoor_const_bigint_value` safe. The helper owns the null/type dispatch and keeps `FromDatum` inside the decoder.
- Removed the now-redundant caller-side unsafe wrappers at the bound-parameter and predicate-value call sites.

Raw planner tree walkers remain unsafe. This packet only converts decoder helpers whose inputs are already a typed `Const` reference or a Datum plus checked OID.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 76 | 73 | -3 |
| `src/` total | 2324 | 2321 | -3 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2321` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2321` direct unsafe blocks under `src/`; packet 030 still requires every row to be removed or residual-registered.
