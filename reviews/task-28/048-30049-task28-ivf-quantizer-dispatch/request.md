# Review Request: Task 28 IVF Quantizer Dispatch Seam

## Summary

This packet addresses packet 30047 feedback seq 02 item 4: the IVF reloption
surface declared multiple quantizer profiles, but build and scan still called
`ProdQuantizer::cached(...)` directly.

Implemented:

- Added an IVF-local enum dispatch layer in `src/am/ec_ivf/quantizer.rs`.
- Routed `auto` and `turboquant` through the existing product-quantizer
  implementation.
- Kept `pq_fastscan` and `rabitq` rejected for v1, using the same unsupported
  storage-format validation as before.
- Routed ecvector build encoding through the dispatch seam.
- Routed scan prepared-query construction, posting payload length, and
  posting-list scoring through the dispatch seam.
- Updated scan debug state to inspect prepared-query details through the enum
  wrapper instead of reaching into `PreparedQuery` directly.

## Scope

This is substrate wiring only. It does not add PQ-FastScan, RaBitQ, DiskANN, or
new recall/latency claims. DiskANN remains task 29.

## Validation

- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - `6 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `77 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`

## Next

Continue item 5 from `plan/tasks/28-ivf-competitive-substrate.md`: reduce
probe-candidate aggregation pressure. After that, re-run the DBPedia 10k/25k
sweep because the optimized rerank path changed the cost profile.
