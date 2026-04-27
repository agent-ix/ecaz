# Task 28 IVF Borrowed Posting Scan

This packet records commit `30d9ffc`, which changes the IVF scan path to visit
posting tuples by reference. Scan no longer decodes every posting into owned
`Vec` fields for payload bytes and heap TIDs; it scores directly from borrowed
page bytes and iterates borrowed heap TID slots. The existing owned posting
visitor remains in place for vacuum/repair paths.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated n64 DBPedia-derived surfaces from packet 30052:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- `ecaz bench latency`, profile `ec_ivf`, `k=10`, `concurrency=1`,
  `iterations=100`, sweep `nprobe=32,48`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| surface | nprobe | packet 30068 p50 | new p50 | packet 30068 p95 | new p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k n64 w25 | 32 | 95.4 ms | 93.4 ms | 104.3 ms | 102.6 ms |
| 10k n64 w25 | 48 | 140.4 ms | 136.1 ms | 157.4 ms | 166.8 ms |
| 25k n64 w25 | 32 | 240.9 ms | 234.2 ms | 254.1 ms | 248.8 ms |
| 25k n64 w25 | 48 | 340.3 ms | 329.9 ms | 357.3 ms | 346.6 ms |

The borrowed scan path is directionally positive across all p50 points and on
three of four p95 points. The larger 25k surface benefits most: p50 improves by
about 2.8% at `nprobe=32` and 3.1% at `nprobe=48` relative to packet 30068.

This still does not close the competitive-latency gap by itself. It removes an
avoidable per-posting allocation/copy cost from the current posting-list layout.

## Artifacts

- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_25k_n64w25_nprobe32_48.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::page --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32`
- `git diff --check`

## Recommendation

Keep this scan decode cleanup. The next Task 28 slice should continue on
posting-list cost: either reduce the number of postings scored per query or
change the posting tuple layout so scan can score denser payload streams with
less per-tuple header work. DiskANN remains task 29 and is not included.
