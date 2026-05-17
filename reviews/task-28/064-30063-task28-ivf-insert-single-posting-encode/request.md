# Task 28 IVF Single-Posting Encode Trial

This packet records a negative live-insert trial. The attempted change added a
live-insert-specific append helper that encoded the one-heap-TID posting tuple
directly, avoiding the temporary `IvfPostingTuple { heaptids: vec![...] }`
construction while keeping the on-disk posting format unchanged.

The code was **not kept**. Focused Rust and PG18 insert tests passed, but the
fresh-database insert stress harness did not improve throughput.

## Measurement Result

The trial was measured on fresh local PG18 database
`task28_ivf_fresh_20260427` with `--require-admin-snapshot`.

| trial | concurrency | inserted rows/s | fresh normalize-once reference | packet 30057 reference |
| --- | ---: | ---: | ---: | ---: |
| single-posting encode | 1 | 267.80 | 273.20 | 275.30 |
| single-posting encode | 4 | 650.20 | 656.20 | 657.50 |

Both runs captured `ec_ivf_index_admin_snapshot` fields and passed the stress
harness. The result suggests the small encode allocation is not the current
nlists=16 live-insert bottleneck.

## Artifacts

- `artifacts/ivf_insert_singleposting_c1.log`
- `artifacts/ivf_insert_singleposting_c4.log`
- `artifacts/manifest.md`

## Validation Performed Before Backout

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_insert`

## Recommendation

Do not land the one-TID append encode helper as an isolated live-insert
optimization. The next useful slice should change a larger structural cost:
posting tuple packing, exact counter maintenance frequency, or another
contention-heavy write path.
