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

Because the long-lived local `postgres` scratch database had an older
extension object set despite reporting extension version `0.1.1`, those first
runs fell back to relation stats for admin fields. A fresh PG18 database
(`task28_ivf_fresh_20260427`) created from the current extension SQL confirmed
the admin snapshot path and drift counters:

| checkpoint | database | concurrency | inserted rows/s | snapshot source | inserted_since_build | reindex_reason |
| --- | --- | ---: | ---: | --- | ---: | --- |
| `647abd1` normalize once | fresh PG18 | 1 | 273.20 | `ec_ivf_index_admin_snapshot` | 2732 | `changed_rows` |
| `647abd1` normalize once | fresh PG18 | 4 | 656.20 | `ec_ivf_index_admin_snapshot` | 6562 | `changed_rows` |

This fresh-database rerun is a measurement-quality follow-up; it does not
change the conclusion that the code cleanup is not a throughput win over packet
30057.

## Artifacts

- `artifacts/ivf_insert_normonce_c1.log`
- `artifacts/ivf_insert_normonce_c4.log`
- `artifacts/ivf_insert_normonce_fresh_c1.log`
- `artifacts/ivf_insert_normonce_fresh_c4.log`
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
