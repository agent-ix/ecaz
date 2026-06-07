# Review Request: IVF TQ+ Real 10k/50k/100k Spread

Task: 86 - TurboVec-Derived TurboQuant Improvements

Implementation commit: `e0ae9fe7dbcfb335cdaa7f47072416e5287ce5a4`

Packet: `reviews/task-86/011-ivf-tqplus-real-spread/`

## What Changed

This packet measures the TurboVec-derived TQ+ calibration candidate against our
own TurboQuant baseline on the production IVF scan path.

The implementation adds an IVF-local `storage_format=turboquant_tqplus` profile
that:

- fits per-coordinate TQ+ `shift` and `scale` calibration from the IVF training
  sample;
- persists calibration metadata in the IVF model chain;
- prepares queries in the calibrated TQ+ space;
- scores packed 4-bit database codes directly with the TQ+ LUT path;
- re-encodes build postings and trained-index inserts with the persisted TQ+
  model;
- rejects model-less TQ+ encode/query paths so plain dispatch cannot silently
  use baseline TurboQuant.

This is still scoped to IVF for measurement. It does not claim HNSW, DiskANN,
or SPIRE support yet.

## Benchmark Shape

Both suites use `ecaz bench suite`, PG18, `ec_ivf`, DBpedia real fixtures,
`rerank=off`, `rerank_width=0`, 200 recall queries, 1000 latency iterations,
and one isolated prefix/index per fixture and storage format.

Configs:

- `suite-baseline.json`: `storage_format=turboquant`
- `suite-tqplus.json`: `storage_format=turboquant_tqplus`

Artifacts and command provenance are in `artifacts/manifest.md`.

## Results

TQ+ improved recall at every measured point, improved p50/p95/p99 latency at
every measured point, and changed index bytes per row only by calibration
metadata noise.

| fixture | nprobe | recall baseline | recall TQ+ | p50 baseline | p50 TQ+ | index B/row baseline | index B/row TQ+ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| real10k | 8 | 0.9740 | 0.9860 | 2.90 ms | 2.68 ms | 951.1 | 952.7 |
| real10k | 24 | 0.9745 | 0.9870 | 7.02 ms | 6.48 ms | 951.1 | 952.7 |
| real10k | 32 | 0.9745 | 0.9870 | 8.96 ms | 8.30 ms | 951.1 | 952.7 |
| real50k | 16 | 0.9265 | 0.9400 | 10.8 ms | 10.0 ms | 925.2 | 925.5 |
| real50k | 48 | 0.9450 | 0.9665 | 31.5 ms | 28.9 ms | 925.2 | 925.5 |
| real50k | 64 | 0.9470 | 0.9685 | 45.2 ms | 41.1 ms | 925.2 | 925.5 |
| real100k | 32 | 0.9225 | 0.9300 | 22.8 ms | 21.2 ms | 925.5 | 925.7 |
| real100k | 96 | 0.9505 | 0.9605 | 70.7 ms | 64.5 ms | 925.5 | 925.7 |
| real100k | 128 | 0.9525 | 0.9620 | 91.5 ms | 83.5 ms | 925.5 | 925.7 |

Full p95/p99 rows are in `artifacts/manifest.md` and the suite-local
`results.jsonl` files.

## Interpretation

TQ+ is no longer just a synthetic probe. On real corpora it is a viable
TurboQuant-family improvement candidate for IVF: better recall, lower scan
latency, and unchanged per-row code size in this lane.

The latency win appears to come from the query/LUT/scoring shape, not from fewer
bytes. The per-row payload remains the packed 4-bit code plus the existing
per-vector scalar. The extra shift/scale calibration is model metadata, not hot
per-row storage.

The result is strong enough to justify follow-up work in two directions:

- promote naming/API out of the `*_for_test` TQ+ helper surface before treating
  this as production-ready shared TurboQuant code;
- port and measure TQ+ on the AMs the task cares about next, starting with
  SPIRE/TurboQuant and then HNSW/DISKANN if the codec adapter surface is clean.

## Validation

- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed
  (`artifacts/cargo-check-pg18.log`).
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::quantizer -- --test-threads=1`: 14 passed
  (`artifacts/cargo-test-ec-ivf-quantizer-single-thread.log`).
- `cargo test -p ecaz --lib --no-default-features --features pg18 metadata_decode_accepts_known_format_codes_and_rejects_unknown_codes`: 1 passed
  (`artifacts/cargo-test-ec-ivf-metadata-format.log`).

## Review Notes

Please focus review on:

- whether reusing the IVF PQ codebook tuple chain for TQ+ shift/scale is
  acceptable as a measurement profile or should be replaced before promotion;
- whether the trained-index insert re-encode path is sufficient for
  `turboquant_tqplus`;
- whether this packet is enough evidence to advance TQ+ to a production-naming
  cleanup and cross-AM measurement task.
