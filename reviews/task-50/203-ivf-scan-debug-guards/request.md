# Task 50 Review Request: IVF Scan Debug Guards

## Summary

This slice narrows the IVF scan pg-test/debug unsafe surface in `src/am/ec_ivf/scan.rs`.

The thin debug wrappers for AM callbacks, executor scan calls, scan opaque reads, heap TID reads, order-by output inspection, and heap OID resolution are now safe guard functions. Each guard owns the local invariant and contains the minimal FFI/raw-pointer access needed for the debug helper. Callers no longer need to propagate unsafe through those adapter signatures.

## Unsafe Burn Down

- `src/am/ec_ivf/scan.rs` unsafe token count: `73 -> 58`
- `src/am/ec_ivf/scan.rs` direct `unsafe { ... }` blocks: `27 -> 27`
- `src/` total unsafe token count after this slice: `2608`

## Code Commit

- `8431f3542faab06296488c008127ec6585449b48` Tighten IVF scan debug guards

## Validation

- `rustfmt --edition 2021 --check src/am/ec_ivf/scan.rs`
  - log: `artifacts/rustfmt-ivf-scan.log`
  - passed with existing stable-rustfmt warnings for unstable import grouping options
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - passed with existing unused-import warning in `src/am/mod.rs`
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
  - log: `artifacts/cargo-test-ec-ivf-no-run.log`
  - passed with existing Hadamard test-helper dead-code warnings
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - passed
- IVF scan unsafe scan:
  - log: `artifacts/ivf-scan-unsafe-scan.log`

