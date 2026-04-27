# Task 28 IVF Combined Insert Stats WAL Trial

This packet records a negative live-insert trial. The attempted change combined
the per-insert list-directory counter rewrite and metadata counter rewrite into
one generic WAL transaction after the posting append, preserving exact counters
and admin snapshot semantics.

The code was **not kept**. Focused Rust and PG18 insert tests passed, but the
fresh-database insert stress harness did not improve throughput.

## Measurement Result

The trial was measured on fresh local PG18 database
`task28_ivf_fresh_20260427` with `--require-admin-snapshot`.

| trial | concurrency | inserted rows/s | fresh normalize-once reference | packet 30057 reference |
| --- | ---: | ---: | ---: | ---: |
| combined stats WAL | 1 | 265.20 | 273.20 | 275.30 |
| combined stats WAL | 4 | 645.10 | 656.20 | 657.50 |

Both runs captured `ec_ivf_index_admin_snapshot` fields and passed the stress
harness. The result argues against landing this combined-WAL rewrite as the
next live-insert lever.

## Artifacts

- `artifacts/ivf_insert_combinedstats_c1.log`
- `artifacts/ivf_insert_combinedstats_c4.log`
- `artifacts/manifest.md`

## Validation Performed Before Backout

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_insert`

## Recommendation

Do not land the combined directory/metadata generic-WAL transaction trial. The
remaining live-insert path still needs a more structural change, likely around
the one-posting-per-row append shape or the need to maintain exact per-list and
metadata counters on every inserted row.
