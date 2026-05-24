# Task 50 Review Request: AM Shared Format Type Name

## Summary

Removed duplicate PostgreSQL `format_type_be`/`pfree` handling from two AM
paths by reusing the existing shared storage helper:

- IVF build now calls `crate::storage::type_info::formatted_base_type_name`
  when resolving the indexed vector kind.
- SPIRE DML frontdoor primary-key metadata now calls the same helper instead
  of carrying its own C-string allocation/free block.

This keeps the raw C-string lifetime contract centralized in
`src/storage/type_info.rs` instead of repeating it at each AM call site.

## Unsafe Burndown

- `src/am/ec_ivf/build.rs` unsafe grep count: `19 -> 18`
- `src/am/ec_spire/dml_frontdoor/mod.rs` unsafe grep count: `69 -> 68`
- repository `src` unsafe grep count: `2443 -> 2441`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_ivf/build.rs src/am/ec_spire/dml_frontdoor/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.
- `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the shared helper is appropriate for both call sites and that the
SPIRE DML frontdoor behavior remains acceptable when formatting the base type
name for the `INT8OID` primary-key metadata path.
