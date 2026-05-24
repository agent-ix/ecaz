# Review Request: SPIRE Scan Datum Boundary

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `0dd6a609bcd9791659c2c6528493b939498cc0c1`

## Summary

This slice collapses SPIRE heap-row source-vector decoding in `src/am/ec_spire/scan/relation.rs`.

- Removed the private `indexed_vector_datum_to_source_vector()` unsafe helper.
- Removed the private `detoasted_varlena_bytes()` unsafe helper.
- Moved the non-null Datum check, `DetoastedVarlena::packed_from_datum`, owned-byte copy, and source-vector decode into the existing heap-row load path.
- Preserved `heap_reader.clear()` after both success and decode-error results by returning a `Result` from the boundary rather than returning from inside it.
- No safe raw-pointer helper signatures were added.

Unsafe count movement:

- `src/am/ec_spire/scan/relation.rs`: 7 -> 5 direct `unsafe {` blocks.
- `src`: 1169 -> 1167 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/scan/relation.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1167 current unsafe rows`.

Artifacts are in `reviews/task-50/378-spire-scan-datum-boundary/artifacts/`.
