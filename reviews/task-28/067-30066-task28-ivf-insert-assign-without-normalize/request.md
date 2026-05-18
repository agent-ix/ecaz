# Task 28 IVF Insert Assignment Without Source Normalization

This packet records the first 1536D live-insert measurement after commit
`302ee78`, which avoids allocating a normalized copy of each inserted source
vector for centroid assignment. The assignment still validates dimension,
finite values, and non-zero norm; it scores the original source against the
already-normalized centroids.

## Measurement Result

Measured on fresh local PG18 database `task28_ivf_fresh_20260427` with
`--require-admin-snapshot`:

| fixture | concurrency | run | inserted rows/s | total inserted | index bytes |
| --- | ---: | --- | ---: | ---: | ---: |
| 1536D synthetic | 1 | r1 | 122.80 | 1228 | 2220032 |
| 1536D synthetic | 1 | r2 | 126.10 | 1261 | 2252800 |
| 1536D synthetic | 4 | r1 | 406.80 | 4068 | 5038080 |
| 1536D synthetic | 4 | r2 | 403.00 | 4030 | 4997120 |

The prior 1536D baseline in packet 30065 reported 124.30 rows/s at c1 and
393.60 rows/s at c4. This checkpoint is effectively neutral at c1 across two
runs and modestly positive at c4. Treat it as an insert-path cleanup with a
small concurrent 1536D win, not as a major throughput lever.

Both reruns captured `ec_ivf_index_admin_snapshot` fields and passed the stress
harness.

## Artifacts

- `artifacts/ivf_insert_assignraw_dim1536_c1.log`
- `artifacts/ivf_insert_assignraw_dim1536_c1_r2.log`
- `artifacts/ivf_insert_assignraw_dim1536_c4.log`
- `artifacts/ivf_insert_assignraw_dim1536_c4_r2.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::training --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_insert`
- `git diff --check`

## Recommendation

Keep this checkpoint, but do not spend more Task 28 time on single-digit insert
micro-optimizations until the reviewer-requested scan path work is complete.
The next IVF slice should stay on the competitive-latency arc: heap rerank
prefetch evidence, index-internal rerank scoring evidence, then a refreshed
recall/latency sweep.
