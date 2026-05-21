# Task 50 Review Request: SPIRE Common PostgreSQL Pointer Helpers

## Summary

Centralized duplicate SPIRE planner/callback pointer helpers into
`src/am/common/pg_ptr.rs`.

The DML frontdoor and custom-scan cost helper code previously each carried
local unsafe wrappers for:

- `PgList::from_pg`
- `ptr.as_ref()`

Both call sites now import the common helpers under their existing local names,
so the call-site contracts stay visible while the duplicated unsafe helper
bodies are removed.

## Unsafe Burndown

- `src/am/ec_spire/dml_frontdoor/mod.rs` unsafe grep count: `68 -> 64`
- `src/am/ec_spire/custom_scan/cost_helpers.rs` unsafe grep count: `26 -> 22`
- new shared helper: `src/am/common/pg_ptr.rs` unsafe grep count: `0 -> 4`
- repository `src` unsafe grep count: `2409 -> 2405`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/common/pg_ptr.rs src/am/common/mod.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/dml_frontdoor/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the shared helpers preserve the PostgreSQL planner/callback
pointer lifetime contract and that importing them under the existing local
names keeps call-site intent clear.
