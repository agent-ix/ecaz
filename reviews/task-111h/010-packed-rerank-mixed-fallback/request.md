# Task 111h Review Request: Packed Rerank Mixed Fallback

Code commit: `fbddda0935a8640facc892a2e02a70a89178e38a`

## Summary

This checkpoint adds PG18 coverage for the packed index-side rerank fallback
path when the survivor frontier cannot rely entirely on posting-carried group
header TIDs.

- Adds a test-only debug scan mode that clears one survivor's rerank group TID
  before the rerank stage, creating a mixed frontier with one missing direct
  group pointer.
- Tracks test-only `rerank_full_chain_loads` in the debug counter snapshot so
  the fixture proves the full-chain loader branch directly rather than
  inferring it from page counts.
- Adds `test_ec_ivf_index_placement_mixed_fallback_chain`, which compares the
  normal direct-pointer scan with the mixed fallback scan and asserts:
  - same output IDs,
  - same scored survivor payload bytes,
  - no f16 batch slab copy,
  - direct path uses zero full-chain loads,
  - mixed path uses exactly one full-chain load.

This covers the mixed missing-direct-pointer part of the lifecycle checklist.
Legacy `0x2A` benchmarking, table-owned payload storage, and the full benchmark
matrix remain open.

## Validation

Artifacts are recorded in `artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`: passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement_mixed_fallback_chain`: passed, 1 test.
