# Task 124 Packet 008 Artifact Manifest

- Task bucket: `reviews/task-124/`
- Packet path: `reviews/task-124/008-tq2-stage2-suite/`
- Head SHA: `ed95ab3e8667ac52549abb0439e8334dd2366280`
- Timestamp: `2026-06-29T03:46:28Z`
- Lane: local PG18, release extension install, `ec_ivf` staged real corpora
- Storage format: `coarse_rerank`
- Coarse frontier: `coarse_format=rabitq`, `coarse_bits=1`, `nlists=64`, recall/latency sweeps at `nprobe=32,64`
- Stage-2 variant: `rerank_placement=index`, `rerank_format=turboquant2`, `rerank_width=100`, `stage2_final_rerank_width=15`
- Fixture isolation: one index/table prefix per scale: `task124_tq2_10k`, `task124_tq2_50k`, `task124_tq2_100k`
- Outcome: not promotable and not Task 124 closeout; TQ2 is a middle-ground payload size between TQ-binary and full TQ, but recall remains far below matched-recall needs and the current TQ2 stage-2 scorer is scalar fallback, not the full TQ4 SIMD lane.

## Code Under Test

Commit `ed95ab3e8667ac52549abb0439e8334dd2366280` adds an experimental `turboquant2` rerank format:

- `src/am/ec_ivf/options.rs`: reloption parse/validation for `turboquant2` and `tq2`.
- `src/am/ec_ivf/quantizer.rs`: TurboQuant encode/query/score paths honor a resolved TQ bit width instead of hard-wiring 4-bit.
- `src/am/ec_ivf/rerank.rs`: `turboquant2` sidecar codec uses 2-bit TQ payloads and scalar fallback for non-4-bit TQ batch APIs.
- `src/am/ec_ivf/scan.rs`: permits stage-2 final exact rerank after this TQ-derived stage-2 format.
- `src/am/ec_ivf/page.rs`: mirrors the new enum value for non-PG builds.

For 1536D, full no-QJL TQ4 sidecar payload is 768 bytes after packet 006 gamma elision. TQ2 sidecar payload is 388 bytes: 384 code bytes plus a 4-byte gamma.

## Validation Commands

All validation below passed before this packet was written:

```text
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::options --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::rerank --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo fmt --check
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

## Suite Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/008-tq2-stage2-suite/artifacts/task124-tq2-final15-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/008-tq2-stage2-suite/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/008-tq2-stage2-suite/artifacts/tq2-final15-suite/suite-manifest.json
```

Report summary: `completed 12`, `failed 0`, `skipped 0`.

## Committed Artifacts

- `task124-tq2-final15-suite.json`: SuiteConfig for the 10k/50k/100k TQ2 matrix.
- `suite-run.log`: suite runner log.
- `tq2-final15-suite/suite-manifest.json`: structured suite manifest.
- `tq2-final15-suite/results.jsonl`: structured result records.
- `tq2-final15-suite/load-*.log`: per-scale load logs.
- `tq2-final15-suite/recall-*.log`: per-scale recall logs.
- `tq2-final15-suite/latency-*.log`: per-scale latency logs with candidate batch counters.
- `tq2-final15-suite/storage-*.log`: per-scale storage logs.

Regenerable `truth-*.json` caches were intentionally not committed.

## Key Results

### Recall at k=10

| Scale | nprobe=32 | nprobe=64 |
| --- | ---: | ---: |
| 10k | 0.9770 | 0.9770 |
| 50k | 0.8050 | 0.8050 |
| 100k | 0.7490 | 0.7550 |

### Latency

| Scale | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.93 ms | 1.05 ms | 1.24 ms |
| 10k | 64 | 1.39 ms | 1.49 ms | 1.69 ms |
| 50k | 32 | 2.57 ms | 2.84 ms | 3.06 ms |
| 50k | 64 | 4.88 ms | 5.06 ms | 5.20 ms |
| 100k | 32 | 5.22 ms | 5.79 ms | 6.16 ms |
| 100k | 64 | 9.53 ms | 9.75 ms | 9.81 ms |

### Storage

| Scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 7.6 MiB | 793.0 B |
| 50k | 34.9 MiB | 732.7 B |
| 100k | 69.3 MiB | 727.0 B |

### Kernel Counters

The only block-kernel counter rows in this suite are the coarse RaBitQ frontier:

| Scale | nprobe | quant | isa | scalar_candidates | width_ge32 |
| --- | ---: | --- | --- | ---: | ---: |
| 10k | 32 | rabitq | neon | 0 | 3391 |
| 10k | 64 | rabitq | neon | 0 | 6700 |
| 50k | 32 | rabitq | neon | 0 | 10389 |
| 50k | 64 | rabitq | neon | 0 | 22000 |
| 100k | 32 | rabitq | neon | 0 | 21443 |
| 100k | 64 | rabitq | neon | 0 | 41300 |

There are no TQ/TQ-QJL block-kernel rows for `turboquant2`; the current implementation uses scalar fallback for non-4-bit TQ stage-2 scoring.

## Comparison

Packet 007 TQ-binary:

- 100k recall: 0.7810 at nprobe32, 0.7980 at nprobe64.
- 100k latency: 4.86 / 5.40 / 5.69 ms at nprobe32; 9.08 / 9.28 / 9.55 ms at nprobe64.
- 100k storage: 46.1 MiB, 483.0 B/row.
- Stage-2 binary scorer was full NEON.

Packet 008 TQ2:

- 100k recall: 0.7490 at nprobe32, 0.7550 at nprobe64.
- 100k latency: 5.22 / 5.79 / 6.16 ms at nprobe32; 9.53 / 9.75 / 9.81 ms at nprobe64.
- 100k storage: 69.3 MiB, 727.0 B/row.
- Stage-2 TQ2 scorer is scalar fallback.

TQ2 improves 10k recall over TQ-binary but is worse than TQ-binary at 50k/100k recall, worse on storage, and not SIMD in the current scorer. It also remains well behind the full-TQ/f32 matched-recall target from earlier packets.

## Decision

Do not promote `turboquant2` as-is. This closes off the naive 2-bit TQ sidecar path:

- It is more informative than a 1-bit sign sidecar at 10k, but still fails recall badly at 50k/100k.
- It is smaller than full TQ4, but too large relative to TQ-binary and f32/source baseline for its quality.
- It is not full SIMD today; making it SIMD would not fix the recall failure.

Next TQ work should focus on preserving the full TQ4 scoring surface while reducing overhead around payload placement/materialization, or on a more selective/fused stage-2 design rather than lower-bit TQ payloads.
