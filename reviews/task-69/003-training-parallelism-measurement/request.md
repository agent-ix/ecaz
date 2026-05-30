# Review Request: Task 69 Packet 003 Training Parallelism Measurement

Code commit: `d8adfbfa51466fccfa1e6401c442283ffb368cd8`

## Summary

This packet adds and runs an ignored release-mode measurement harness for Task 69 Slice D. The harness measures scalar vs parallel common-training implementations at Task 68-shaped training sizes and asserts byte-equality before emitting timing lines.

Artifact manifest: `artifacts/manifest.md`

Rollup: `artifacts/measurement-summary.md`

## Validation

Formatting:

```text
cargo fmt --check
```

Result: passed. Stable rustfmt prints the repo's existing warnings about unstable formatting config keys.

Focused tests:

```text
cargo test -p ecaz --lib am::common::training --no-default-features --features pg18
```

Result:

```text
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 1921 filtered out
```

## Release Measurement

Default Rayon pool, 18 threads:

| kind | shape | scalar_ms | parallel_ms | speedup | digest equality |
| --- | --- | ---: | ---: | ---: | --- |
| kmeans | `spire_10k_nlists32` | 1716.896 | 147.842 | 11.613x | yes |
| kmeans | `spire_100k_sample10k_nlists128` | 6662.520 | 484.940 | 13.739x | yes |
| grouped_pq4 | `ivf_pq_fastscan_10k` | 137.797 | 11.645 | 11.834x | yes |

`RAYON_NUM_THREADS=1`:

| kind | shape | scalar_ms | parallel_ms | parallel/scalar | regression |
| --- | --- | ---: | ---: | ---: | ---: |
| kmeans | `spire_10k_nlists32` | 1695.591 | 1681.713 | 0.9918 | -0.8% |
| kmeans | `spire_100k_sample10k_nlists128` | 6656.276 | 6633.730 | 0.9966 | -0.3% |
| grouped_pq4 | `ivf_pq_fastscan_10k` | 135.971 | 133.147 | 0.9792 | -2.1% |

Worst observed single-thread regression: none. All three parallel paths were slightly faster than scalar with `RAYON_NUM_THREADS=1`.

## Notes

- The `spire_100k_sample10k_nlists128` shape uses 10,000 training rows because both IVF and SPIRE cap auto training samples at 10,000 for larger corpora.
- The grouped-PQ shape uses 10,000 rows, 1,536 dimensions, `group_size=16`, and 3 iterations, matching the IVF PQ-FastScan default group-size and k-means iteration path.
- Digest values are 64-bit FNV-1a over the returned model floats. They are duplicated as `digest` and `parallel_digest` in the raw logs; equality is also enforced with `assert_eq!` inside the harness.

## Reviewer Ask

Please verify that this satisfies Slice D: multi-x release speedup, byte-equal model output, and no `RAYON_NUM_THREADS=1` overhead regression.
