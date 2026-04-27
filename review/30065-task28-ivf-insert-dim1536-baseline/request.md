# Task 28 IVF Insert 1536D Baseline

This packet records a 1536D live-insert baseline using the dimension-aware
`ecaz stress ivf-insert` harness from commit `656b2dc`.

The goal is to separate the 4D fixture's generic write-path overhead from
production-like dimensional work in centroid assignment, encoding, and posting
payload writes.

## Measurement Result

Measured on fresh local PG18 database `task28_ivf_fresh_20260427` with
`--require-admin-snapshot`:

| fixture | concurrency | inserted rows/s | total inserted | index bytes |
| --- | ---: | ---: | ---: | ---: |
| 1536D synthetic | 1 | 124.30 | 1243 | 2236416 |
| 1536D synthetic | 4 | 393.60 | 3936 | 4898816 |

For context, packet 30060's fresh 4D normalize-once runs reported 273.20 rows/s
at c1 and 656.20 rows/s at c4. The 1536D path remains concurrent, but dimension
cost is now a large part of live-insert throughput.

Both 1536D runs captured `ec_ivf_index_admin_snapshot` fields and passed the
stress harness.

## Artifacts

- `artifacts/ivf_insert_dim1536_c1.log`
- `artifacts/ivf_insert_dim1536_c4.log`
- `artifacts/manifest.md`

## Validation

Measurement command validation came from packet 30064:

- `cargo fmt --check`
- `cargo test -p ecaz-cli ivf_insert`
- 1536D one-second PG18 smoke with `--require-admin-snapshot`

This packet adds the 10-second c1/c4 measurement runs only.

## Recommendation

Stop treating the 4D insert stress fixture as sufficient for live-insert
optimization decisions. The next implementation slice should target
dimension-dependent work first: avoid re-normalizing/allocating 1536D source
vectors where possible, then revisit centroid model caching under high
dimensions rather than nlists=16/4D alone.
