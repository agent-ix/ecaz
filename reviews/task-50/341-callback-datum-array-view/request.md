# Review Request: Callback Datum Array View

Task: 50 unsafe burndown

Commit under review:

- `e24a59b7` - `Centralize callback datum array reads`

## Summary

This packet centralizes build-callback `values` / `isnull` array reads.

- Adds `am::common::pg_ptr::DatumArrayView`, built from checked `NonNull<Datum>` and `NonNull<bool>` arrays.
- Updates HNSW build tuple decoding to read the indexed datum through the view.
- Updates SPIRE build tuple decoding and source-identity INCLUDE column decoding to use the same view.
- Removes repeated direct `unsafe { *isnull }`, `unsafe { *values }`, and offset-array dereferences from the touched build paths.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1320` to `1315`.
- The targeted callback datum/isnull raw dereference pattern no longer appears under the AM directories scanned by the packet artifact.
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1315` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `DatumArrayView` does not reintroduce a safe raw-pointer API.
- Confirm HNSW and SPIRE build tuple NULL/error behavior remains equivalent.
