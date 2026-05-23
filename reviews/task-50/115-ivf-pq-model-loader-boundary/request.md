# Task 50 Review Request: IVF PQ Model Loader Boundary

## Summary

This slice removes a reducible unsafe contract from the IVF PQ/FastScan model-loading path used by insert and scan code.

Code commit: `9cffbea32d5964fb2ea058bb8c8f338f0a9100e4`

## Changes

- Converted `quantizer::load_pq_fastscan_model` from `unsafe fn` to a safe function.
- Removed the corresponding unsafe call blocks from IVF insert-time reencoding and scan-time PQ/FastScan model initialization.

The loader itself validates storage format, codebook head, group size, transform divisibility, codebook order, and chain termination before returning the model. The page read it delegates to is already a safe API, so the whole-function unsafe contract was not carrying a distinct FFI invariant.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/115-ivf-pq-model-loader-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1665` across `124` files.
- Ledger check passed: `ledger covers 1665 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
