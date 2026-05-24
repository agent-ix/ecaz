# Task 50 Review Request: AM Remove Forwarding Unsafe Wrappers

## Summary

Removed redundant top-level unsafe forwarding wrappers from `src/am/mod.rs`.

The affected HNSW, IVF, and DiskANN diagnostic/snapshot functions are now
crate-visible alias re-exports of the underlying unsafe AM functions. This
keeps the same unsafe call contract for SQL-facing callers while deleting
wrappers that only repeated the contract and forwarded immediately.

Also removed now-unused snapshot type re-exports from `ec_ivf/mod.rs` and
`ec_diskann/mod.rs` so the change does not introduce new unused-import
warnings.

## Unsafe Burndown

- `src/am/mod.rs` unsafe grep count: `20 -> 2`
- repository `src` unsafe grep count: `2432 -> 2414`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/mod.rs src/am/ec_diskann/mod.rs src/am/ec_ivf/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the alias re-exports preserve the `am::...` names used by
SQL-facing code and tests while removing only redundant forwarding layers.
