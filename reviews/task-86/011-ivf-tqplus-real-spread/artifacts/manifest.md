# Task 86 Packet 011 Artifact Manifest

Head SHA: `e0ae9fe7dbcfb335cdaa7f47072416e5287ce5a4`

Task bucket: `reviews/task-86/011-ivf-tqplus-real-spread`

Timestamp: 2026-06-07 14:30 PDT

Note: benchmark artifacts were generated from the working tree now committed as
`e0ae9fe7dbcfb335cdaa7f47072416e5287ce5a4`.

## Lane

- Access method: `ec_ivf`
- Fixture spread: real10k, real50k, real100k
- Dimensions: DBpedia staged fixture, 1536-dimensional `ecvector`
- Storage formats: baseline `turboquant`, candidate `turboquant_tqplus`
- Rerank mode: `rerank=off`, `rerank_width=0`
- Reason for rerank off: measure approximate TurboQuant-family scoring directly; heap rerank would mask scorer/ranking differences.
- Query count: 200 recall queries, 1000 latency iterations
- Sweep:
  - real10k: nprobe `8,24,32`, nlists `32`
  - real50k: nprobe `16,48,64`, nlists `64`
  - real100k: nprobe `32,96,128`, nlists `128`
- Surface isolation: one prefix per fixture and storage format, one IVF index per prefix.

## Commands

```text
./target/debug/ecaz bench suite audit --config reviews/task-86/011-ivf-tqplus-real-spread/suite-baseline.json
./target/debug/ecaz bench suite audit --config reviews/task-86/011-ivf-tqplus-real-spread/suite-tqplus.json
./target/debug/ecaz dev install --log-file reviews/task-86/011-ivf-tqplus-real-spread/artifacts/install-pg18-tqplus.log ecaz-pg-test --pg 18
./target/debug/ecaz bench suite run --config reviews/task-86/011-ivf-tqplus-real-spread/suite-baseline.json --host /Users/peter/.pgrx --port 28818
./target/debug/ecaz bench suite run --config reviews/task-86/011-ivf-tqplus-real-spread/suite-tqplus.json --host /Users/peter/.pgrx --port 28818
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::quantizer -- --test-threads=1
cargo test -p ecaz --lib --no-default-features --features pg18 metadata_decode_accepts_known_format_codes_and_rejects_unknown_codes
```

## Artifact Index

- `suite-baseline.json`: baseline TurboQuant suite config.
- `suite-tqplus.json`: TQ+ suite config.
- `artifacts/baseline/suite-manifest.json`: completed baseline suite manifest.
- `artifacts/baseline/results.jsonl`: completed baseline structured results.
- `artifacts/baseline/*.log`: baseline load, storage, recall, and latency logs.
- `artifacts/tqplus/suite-manifest.json`: completed TQ+ suite manifest.
- `artifacts/tqplus/results.jsonl`: completed TQ+ structured results.
- `artifacts/tqplus/*.log`: TQ+ load, storage, recall, and latency logs.
- `artifacts/truth-real10k-k10.json`, `artifacts/truth-real50k-k10.json`, `artifacts/truth-real100k-k10.json`: packet-local truth caches.
- `artifacts/install-pg18-tqplus.log`: PG18 extension install log.
- `artifacts/cargo-check-pg18.log`: PG18 library compile validation.
- `artifacts/cargo-test-ec-ivf-quantizer-single-thread.log`: IVF quantizer unit validation, including TQ+ dispatch.
- `artifacts/cargo-test-ec-ivf-metadata-format.log`: metadata storage-format decode validation.
- `artifacts/cargo-check-pg18-after-format-plan.log`: PG18 library compile validation after TQ+ production-naming, quantile-cache, and calibration-validation cleanup.
- `artifacts/cargo-test-ec-ivf-quantizer-single-thread-after-format-plan.log`: IVF quantizer unit validation after cleanup.
- `artifacts/cargo-test-ec-ivf-metadata-format-after-format-plan.log`: metadata storage-format decode validation after cleanup.
- `artifacts/tqplus-format-plan.md`: task-local format plan for the new IVF `turboquant_tqplus` storage-format tag and calibration chain.

## Key Results

### Storage

| fixture | baseline index B/row | TQ+ index B/row | delta |
| --- | ---: | ---: | ---: |
| real10k | 951.1 | 952.7 | +1.6 |
| real50k | 925.2 | 925.5 | +0.3 |
| real100k | 925.5 | 925.7 | +0.2 |

### Recall@10

| fixture | nprobe | baseline | TQ+ | delta |
| --- | ---: | ---: | ---: | ---: |
| real10k | 8 | 0.9740 | 0.9860 | +0.0120 |
| real10k | 24 | 0.9745 | 0.9870 | +0.0125 |
| real10k | 32 | 0.9745 | 0.9870 | +0.0125 |
| real50k | 16 | 0.9265 | 0.9400 | +0.0135 |
| real50k | 48 | 0.9450 | 0.9665 | +0.0215 |
| real50k | 64 | 0.9470 | 0.9685 | +0.0215 |
| real100k | 32 | 0.9225 | 0.9300 | +0.0075 |
| real100k | 96 | 0.9505 | 0.9605 | +0.0100 |
| real100k | 128 | 0.9525 | 0.9620 | +0.0095 |

### Latency p50/p95/p99

| fixture | nprobe | baseline p50 | TQ+ p50 | baseline p95 | TQ+ p95 | baseline p99 | TQ+ p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| real10k | 8 | 2.90 ms | 2.68 ms | 3.11 ms | 2.84 ms | 3.27 ms | 2.90 ms |
| real10k | 24 | 7.02 ms | 6.48 ms | 7.49 ms | 6.83 ms | 7.92 ms | 7.12 ms |
| real10k | 32 | 8.96 ms | 8.30 ms | 9.27 ms | 8.60 ms | 10.2 ms | 9.32 ms |
| real50k | 16 | 10.8 ms | 10.0 ms | 12.2 ms | 11.3 ms | 12.6 ms | 11.7 ms |
| real50k | 48 | 31.5 ms | 28.9 ms | 34.1 ms | 31.3 ms | 36.1 ms | 32.7 ms |
| real50k | 64 | 45.2 ms | 41.1 ms | 46.1 ms | 41.9 ms | 48.5 ms | 44.4 ms |
| real100k | 32 | 22.8 ms | 21.2 ms | 25.1 ms | 23.4 ms | 26.2 ms | 24.5 ms |
| real100k | 96 | 70.7 ms | 64.5 ms | 73.7 ms | 67.3 ms | 76.3 ms | 68.3 ms |
| real100k | 128 | 91.5 ms | 83.5 ms | 92.8 ms | 84.3 ms | 98.4 ms | 86.2 ms |

## Validation

- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::quantizer -- --test-threads=1`: 14 passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 metadata_decode_accepts_known_format_codes_and_rejects_unknown_codes`: 1 passed.
- After production-naming/format-plan cleanup:
  - `cargo check -p ecaz --lib --no-default-features --features pg18`: passed.
  - `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::quantizer -- --test-threads=1`: 14 passed.
  - `cargo test -p ecaz --lib --no-default-features --features pg18 metadata_decode_accepts_known_format_codes_and_rejects_unknown_codes`: 1 passed.

An earlier parallel quantizer-filter run failed the RaBitQ cache construction
counter test because that test uses global cache instrumentation. The same
filter passed when rerun single-threaded and is the validation result cited
above.

## Format Plan

The implementation adds durable IVF storage-format tag `4` for
`turboquant_tqplus`. The task-local format plan is recorded in
`artifacts/tqplus-format-plan.md`. It documents the tag assignment, calibration
chain layout, compatibility behavior, insert/scan/vacuum semantics, and the
promotion requirements before this measurement profile becomes a broader
production API.
