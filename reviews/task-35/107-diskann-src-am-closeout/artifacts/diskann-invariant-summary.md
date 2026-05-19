# DiskANN Unsafe-Comment Invariant Summary

## PostgreSQL Callback Boundaries

DiskANN AM callbacks now document the borrowed PostgreSQL pointers they dereference during callback execution. The covered boundaries include build, insert, scan, vacuum, cost, options, and routine entry points. Comments identify when `Relation`, `IndexInfo`, callback state, tuple slots, and item pointers are provided by PostgreSQL for the duration of the callback.

## Page And WAL Boundaries

DiskANN page writes now document the buffer/page ownership model before page initialization, item insertion, special-space writes, and generic WAL finalization. The closeout artifacts verify the unsafe-comment baseline no longer lists DiskANN page, metadata, or data-chain writes.

## Vector Datum Boundaries

Build and insert paths now document ecvector Datum detoasting, varlena byte alignment, and `f32` slice construction. The comments tie the raw byte views to the ecvector layout checks that precede conversion.

## SIMD Boundaries

The source-vector inner-product kernels now document target-feature dispatch, vector lane loads/stores, and scalar tail unchecked reads. The AVX2/FMA and NEON loops use the minimum input slice length as the bounds proof for both operands.

## Residual Work

No `src/am` unsafe-comment baseline entries remain. The remaining Task 35 work is the test-only baseline under `src/tests/`.
