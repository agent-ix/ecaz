# Task 50 Packet 208: IVF Debug Helper Safe Surface

## Summary

This slice removes the `unsafe fn` contract from the IVF pg-test/debug helper surface. The helpers already open the needed relation/scan guards or own the local debug callback state; their callers should not have to repeat `unsafe` at every test/debug call site.

Updated surfaces:

- `src/am/ec_ivf/scan.rs` debug helpers now expose safe functions.
- `src/am/ec_ivf/insert.rs` duplicate-heap-TID debug helper now exposes a safe function.
- `src/am/ec_ivf/vacuum.rs` vacuum debug helpers now expose safe functions.
- `src/tests/ec_ivf.rs` and `src/tests/mod.rs` no longer wrap IVF debug helpers in caller-side unsafe blocks.

The remaining unsafe is localized around the actual PostgreSQL metadata reads, scan callbacks, vacuum callbacks, and page/directory readers.

## Counts

- `src`: `2575 -> 2566` unsafe references.
- IVF debug helper unsafe function contracts: `13 -> 0`.
- `src/am/ec_ivf/scan.rs`: `58 -> 50` unsafe references.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_ivf/scan.rs src/am/ec_ivf/insert.rs src/am/ec_ivf/vacuum.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
