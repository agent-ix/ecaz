# Review Request: Callback Heap TID Decoder

Task: 50 unsafe burndown

Commit under review:

- `44aca7d5` - `Centralize callback heap TID decoding`

## Summary

This packet centralizes a repeated callback heap TID pointer-copy pattern.

- Adds `am::common::pg_ptr::item_pointer`, which copies a non-null PostgreSQL `ItemPointerData` into the owned storage `ItemPointer` representation.
- Updates IVF, SPIRE, HNSW, and DiskANN build heap-TID decoders to pass a checked `NonNull<ItemPointerData>` instead of dereferencing `*tid` at each call site.
- Removes the duplicated caller-side `item_pointer_get_both(unsafe { *tid })` pattern from the touched AM build paths.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1327` to `1324`.
- The targeted callback heap TID dereference pattern no longer appears in:
  - `src/am/ec_ivf/build.rs`
  - `src/am/ec_spire/build/tuples.rs`
  - `src/am/ec_hnsw/shared.rs`
  - `src/am/ec_diskann/ambuild.rs`
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1324` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `NonNull<ItemPointerData>` is an acceptable safe boundary for the shared TID decoder.
- Confirm the decoded `ItemPointer` semantics are unchanged for the four AM build paths.
