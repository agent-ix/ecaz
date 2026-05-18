# Review Request: Task 28 IVF Rerank Prefetch and Internal Score

## Summary

This packet starts the competitive-substrate follow-up from packet 30047
feedback seq 02.

Implemented:

- PG18 IVF `heap_f32` rerank now pre-reads candidate heap blocks with
  `ReadStream` before tuple fetch and exact scoring.
- Non-PG18 builds fall back to `PrefetchBuffer` for the same candidate heap
  blocks.
- IVF `heap_f32` rerank now uses an index-internal negative inner-product
  helper backed by the existing source-vector AVX2+FMA/fallback kernel.
- The SQL-stable `negative_inner_product` helper remains unchanged for callers
  that require sequential f32 accumulation.
- Added `plan/tasks/28-ivf-competitive-substrate.md` to track the remaining
  optimization and substrate work before any product benchmark.

## Scope

This is a code-path optimization and planning checkpoint only. It introduces no
new latency or recall claim. A post-optimization sweep is still required before
updating the IVF frontier.

DiskANN remains task 29 and is not included.

## Validation

- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - `6 passed; 0 failed`
- `cargo test --lib negative_inner_product_index_internal --no-default-features --features pg18`
  - `1 passed; 0 failed`
- `git diff --check`

## Next

Run the broader PG18 IVF gate after the remaining small substrate slices, then
re-run the DBPedia 10k/25k `nlists x nprobe x rerank_width` sweep against the
optimized cost profile.
