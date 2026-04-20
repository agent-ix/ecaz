# Task: NaN Gamma Validation

Review: `review/10050-nan-gamma-validation/request.md`
Priority: batch 1
Status: ready

## Prompt

Add a finite-float check for the gamma value in both the build and insert paths.

Problem: if a tqvector datum contains NaN as its gamma value, NaN propagates
through hnsw_rs distance calculations during build (producing undefined priority
queue ordering) and through search scoring (making the node unreachable by
ordered traversal since NaN fails all comparisons). A NaN-gamma node at a high
HNSW level would break greedy descent.

Fix locations:

1. `src/am/build.rs` — function `build_heap_tuple` (line 231). After unpacking the
   tqvector at line 258 where gamma is extracted:

   ```rust
   let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
       .unwrap_or_else(|e| pgrx::error!("ec_hnsw ambuild found invalid tqvector: {e}"));
   ```

   Add immediately after:

   ```rust
   if !gamma.is_finite() {
       pgrx::error!("ec_hnsw does not support non-finite gamma values");
   }
   ```

   There is also a second unpack path in `build_heap_tuple_with_source` (line 273)
   that extracts gamma at line 292. Add the same check there.

2. `src/am/insert.rs` — function `ec_hnsw_aminsert` (line 9). The gamma value comes
   from the tuple returned by `build::build_heap_tuple` at line 22. If you add the
   check in `build_heap_tuple` itself, the insert path is already covered. But
   verify this by tracing the call — if there's any path where gamma could bypass
   `build_heap_tuple`, add an explicit check.

This is a defensive validation. Gamma comes from the quantizer and should always
be finite, but the check is cheap and prevents silent graph corruption.

## Validate

```bash
cargo test
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Branch from current upstream main. Push branch for review.
