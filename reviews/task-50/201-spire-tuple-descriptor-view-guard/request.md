# Task 50 Review Request: SPIRE Tuple Descriptor View Guard

## Summary

This packet adds a reusable `TupleDescView` next to the tuple slot reader/writer helpers and moves SPIRE custom-scan DML plus DML frontdoor tuple descriptor attribute walking onto that view.

The view centralizes:

- tuple descriptor null validation
- `TupleDescAttr`
- dropped-attribute filtering
- PostgreSQL `NameData` to Rust `String` decoding
- copied attribute metadata needed by SPIRE payload and frontdoor code

After this change, the target SPIRE files no longer perform direct `TupleDescAttr` or attribute-name pointer decoding. The descriptor boundary scan shows those operations are isolated in `src/am/common/heap_slot.rs`.

## Code Under Review

- Code commit: `a5ec17c5 Add tuple descriptor view guard`
- Files changed:
  - `src/am/common/heap_slot.rs`
  - `src/am/ec_spire/custom_scan/dml.rs`
  - `src/am/ec_spire/custom_scan/mod.rs`
  - `src/am/ec_spire/custom_scan/tuple_payload.rs`
  - `src/am/ec_spire/dml_frontdoor/mod.rs`

## Unsafe Ledger

- Touched files combined: `unsafe` matches `155 -> 154`
- `src/`: `unsafe` matches `2624 -> 2623`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/common/heap_slot.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/dml_frontdoor/mod.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_spire::custom_scan --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::ec_spire::dml_frontdoor --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass
- Descriptor boundary scan: direct descriptor attr/name operations remain only in `src/am/common/heap_slot.rs` for the target files

## Review Focus

- Confirm `TupleDescView` preserves dropped-attribute filtering and copies the attribute metadata SPIRE needs.
- Confirm custom-scan payload column projection still respects narrowed target lists.
- Confirm DML frontdoor primary-key and column discovery still uses attnum/name/type metadata equivalent to the previous copied `FormData_pg_attribute` path.
