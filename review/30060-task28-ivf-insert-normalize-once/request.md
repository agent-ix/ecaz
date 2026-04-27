# Task 28 IVF Insert Normalize-Once Cleanup

This packet records a narrow live-insert cleanup in commit `647abd1`
(`ivf: avoid duplicate insert normalization`).

`aminsert` previously normalized the source vector in `validate_insert_tuple`
and then normalized it again immediately inside
`training::assign_vector_to_centroid`. The committed change removes the
validation-time normalization and keeps the assignment-time normalization as the
single source of dimension, finite-value, and zero-norm validation before any
posting-page mutation.

## Measurement Result

The 10-second synthetic PG18 insert harness did **not** show a throughput win
over packet 30057. Treat this as a small quality cleanup, not a performance
advance.

| checkpoint | concurrency | inserted rows/s | packet 30057 reference |
| --- | ---: | ---: | ---: |
| `647abd1` normalize once | 1 | 261.00 | 275.30 |
| `647abd1` normalize once | 4 | 649.70 | 657.50 |

Both runs passed the harness and wrote packet-local raw logs.

## Artifacts

- `artifacts/ivf_insert_normonce_c1.log`
- `artifacts/ivf_insert_normonce_c4.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_insert`
- `git diff --check`

## Recommendation

Keep the cleanup because it removes duplicate hot-path validation without
relaxing insert correctness, but do not count it toward the live-insert
throughput target. Continue with the remaining per-row write work: posting
append shape, list-directory updates, and metadata counter updates.
