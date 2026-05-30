# Review Request: Task 69 Packet 001 Common Training Parallelism

Code commit: `578d8f402281b3493c97c1962dab0f9c406bddd1`

## Summary

This packet lands Task 69 slices A-C:

- `train_spherical_kmeans` now uses a rayon parallel normalization and nearest-centroid pass, while preserving source-order f32 accumulation for byte-identical output against the scalar reference.
- `train_grouped_pq4_model` now parallelizes the independent per-group codebook training work and parallelizes SRHT transform application.
- Added `assign_vectors_to_centroids(error_label, sources, model)` and migrated the existing IVF and SPIRE build-time per-vector assignment loops to it.

The old scalar implementations are retained under `#[cfg(test)]` as private references for deterministic equivalence tests.

## Validation

Artifact manifest: `artifacts/manifest.md`

Focused PG18 unit validation:

```text
cargo test -p ecaz --lib am::common::training --no-default-features --features pg18
```

Result:

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1921 filtered out; finished in 0.02s
```

The new tests cover:

- 16 seed/shape combinations for byte-equal spherical k-means output vs scalar.
- 8 seed/shape combinations for byte-equal grouped PQ4 model output vs scalar.
- deterministic lowest-index error reporting in batch assignment.

## Notes For Reviewer

The k-means implementation intentionally keeps accumulation sequential after parallel assignment. That preserves the exact f32 addition order used by the scalar baseline; a per-thread partial-sum reduction would be faster but would not be bit-identical under normal f32 associativity.

No new `unsafe { ... }` blocks are introduced.
