# Task 124 Packet 004 Artifact Manifest

- head SHA: `d542e0faec2f2799f12b5ee0ee1fa2f1fc9302aa`
- task bucket: `reviews/task-124/004-tq-rerank-group-width`
- timestamp: `2026-06-29T02:06:00Z`
- lane: local PG18 release build on `/Users/peter/.pgrx`, database `tqvector_bench`
- fixture: `data/staged-current/ec_real_100k_{corpus,queries}.tsv`
- runner: `ecaz bench suite`
- suite config: `artifacts/task124-tq-group-width-100k-suite.json`
- suite manifest: `artifacts/group-width-100k-manifest.json`
- suite results: `artifacts/group-width-100k-results.jsonl`
- run log: `artifacts/group-width-100k-run.log`
- isolation: one index per table/prefix

## Code Under Review

Code commit `d542e0faec2f2799f12b5ee0ee1fa2f1fc9302aa` adds the build-time IVF reloption `rerank_group_width`.

- `0` preserves the prior behavior: compact rerank sidecar groups flush at `rerank_width`.
- Nonzero values tune the compact index-side rerank sidecar group width without changing the scan frontier width.
- Validation restricts nonzero `rerank_group_width` to `storage_format='coarse_rerank'`, `rerank_placement='index'`, and compact rerank formats.
- The CLI `ec_ivf` profile now recognizes `rerank_group_width` and `stage2_final_rerank_width` so suite runs do not warn on these Task 124 reloptions.

## Commands

Validation commands run before the suite:

- `cargo check -p ecaz --lib --no-default-features --features pg18` passed.
- `cargo test -p ecaz am::ec_ivf::options --lib --no-default-features --features pg18` passed: 27 passed.
- `cargo test -p ecaz am::ec_ivf::build --lib --no-default-features --features pg18` passed: 16 passed.
- `cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18` passed: 29 passed.
- `cargo fmt --check` passed with the repository's existing stable-rustfmt warnings about nightly-only import settings.
- `cargo build --release -p ecaz` passed.
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` passed.
- `cargo check -p ecaz-cli` passed after the CLI known-reloption cleanup.

Suite command:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_bench bench suite run --config reviews/task-124/004-tq-rerank-group-width/artifacts/task124-tq-group-width-100k-suite.json --artifact-dir reviews/task-124/004-tq-rerank-group-width/artifacts --manifest-output reviews/task-124/004-tq-rerank-group-width/artifacts/group-width-100k-manifest.json --results-output reviews/task-124/004-tq-rerank-group-width/artifacts/group-width-100k-results.jsonl --log-file reviews/task-124/004-tq-rerank-group-width/artifacts/group-width-100k-run.log
```

## 100k Recall And Latency

All variants use `nlists=64`, `nprobe=32/64`, `rerank_width=100`, and 100 query recall / 100 query latency samples. TQ variants use `stage2_final_rerank_width=25`.

| variant | nprobe | recall@10 | latency p50 | latency p95 | latency p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| f32 source baseline | 32 | 0.9730 | 5.04 ms | 5.70 ms | 5.79 ms |
| f32 source baseline | 64 | 1.0000 | 9.23 ms | 9.51 ms | 9.73 ms |
| TQ group 100 | 32 | 0.9730 | 5.17 ms | 5.72 ms | 6.09 ms |
| TQ group 100 | 64 | 1.0000 | 9.30 ms | 9.49 ms | 9.82 ms |
| TQ group 32 | 32 | 0.9730 | 5.09 ms | 5.74 ms | 6.40 ms |
| TQ group 32 | 64 | 1.0000 | 9.22 ms | 9.53 ms | 9.80 ms |
| TQ group 16 | 32 | 0.9730 | 5.03 ms | 5.53 ms | 5.85 ms |
| TQ group 16 | 64 | 1.0000 | 9.15 ms | 9.44 ms | 10.3 ms |

## 100k Storage

| variant | index size |
| --- | ---: |
| f32 source baseline | 22.5 MiB |
| TQ group 100 | 100.8 MiB |
| TQ group 32 | 120.1 MiB |
| TQ group 16 | 120.2 MiB |

## TQ SIMD Counters

The TQ stage2 scorer remains fully SIMD in this suite.

| variant | nprobe | quant | candidates | flushes | scalar candidates | width_ge32 |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| TQ group 100 | 32 | turboquant | 10000 | 100 | 0 | 100 |
| TQ group 100 | 64 | turboquant | 10000 | 100 | 0 | 100 |
| TQ group 32 | 32 | turboquant | 10000 | 100 | 0 | 100 |
| TQ group 32 | 64 | turboquant | 10000 | 100 | 0 | 100 |
| TQ group 16 | 32 | turboquant | 10000 | 100 | 0 | 100 |
| TQ group 16 | 64 | turboquant | 10000 | 100 | 0 | 100 |

## Payload Locality Counters At 100k / nprobe64

| variant | group header pages | segment pages | header payload bytes | segment payload bytes | payload bytes scored | decode us | explain execution |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| TQ group 100 | 45 | 216 | 274449 | 1748632 | 77200 | 496 | 13.926 ms |
| TQ group 32 | 67 | 87 | 500916 | 653382 | 77200 | 485 | 13.347 ms |
| TQ group 16 | 75 | 33 | 577346 | 146982 | 77200 | 426 | 12.723 ms |

## Interpretation

This slice confirms a real sidecar locality knob: smaller groups reduce segment reads from 216 pages / 1.75 MiB to 33 pages / 147 KiB for the same 77.2 KiB scored payload. It does not close Task 124 because the latency gain is not clean across p95/p99 and storage gets worse relative to the default TQ layout. The remaining bottleneck is broader coarse-scan and sidecar/header storage overhead, not scalar TQ scoring.

The required 10k / 50k / 100k closeout matrix is intentionally not claimed by this packet; this is a 100k decision sweep to decide whether `rerank_group_width` should become the next promoted TQ optimization.
