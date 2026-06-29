# Task 124 Packet 008: TQ2 Stage-2 Suite

This is a Task 124 TurboQuant-only optimization checkpoint, not closeout.

## Summary

I added an experimental `turboquant2` rerank format to test a middle ground between full 4-bit TQ and the failed 1-bit TQ-binary sidecar from packet 007.

The implementation persists a 2-bit TQ-derived sidecar and allows the existing stage-2 final exact f32 rerank pipeline to run on top of it. For 1536D, the TQ2 sidecar is 388 bytes per row versus 768 bytes for full no-QJL TQ4 after packet 006 gamma elision.

The result is not promotable: TQ2 still loses too much recall at 50k/100k, has worse storage than TQ-binary, and does not use the full TQ SIMD scorer today.

## Code Commit

- `ed95ab3e8667ac52549abb0439e8334dd2366280` - `Add TQ2 stage2 rerank format`

Touched files:

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/quantizer.rs`
- `src/am/ec_ivf/rerank.rs`
- `src/am/ec_ivf/scan.rs`

## Validation

Passed:

```text
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::options --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::rerank --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo fmt --check
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

## Benchmark Evidence

Suite config:

- `reviews/task-124/008-tq2-stage2-suite/artifacts/task124-tq2-final15-suite.json`

Suite command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/008-tq2-stage2-suite/artifacts/task124-tq2-final15-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/008-tq2-stage2-suite/artifacts/suite-run.log
```

Artifacts:

- `reviews/task-124/008-tq2-stage2-suite/artifacts/manifest.md`
- `reviews/task-124/008-tq2-stage2-suite/artifacts/suite-run.log`
- `reviews/task-124/008-tq2-stage2-suite/artifacts/tq2-final15-suite/suite-manifest.json`
- `reviews/task-124/008-tq2-stage2-suite/artifacts/tq2-final15-suite/results.jsonl`

Suite completed 12 steps with 0 failures.

## Results

Recall fails matched-recall requirements:

| Scale | nprobe=32 | nprobe=64 |
| --- | ---: | ---: |
| 10k | 0.9770 | 0.9770 |
| 50k | 0.8050 | 0.8050 |
| 100k | 0.7490 | 0.7550 |

Latency:

| Scale | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.93 ms | 1.05 ms | 1.24 ms |
| 10k | 64 | 1.39 ms | 1.49 ms | 1.69 ms |
| 50k | 32 | 2.57 ms | 2.84 ms | 3.06 ms |
| 50k | 64 | 4.88 ms | 5.06 ms | 5.20 ms |
| 100k | 32 | 5.22 ms | 5.79 ms | 6.16 ms |
| 100k | 64 | 9.53 ms | 9.75 ms | 9.81 ms |

Storage:

| Scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 7.6 MiB | 793.0 B |
| 50k | 34.9 MiB | 732.7 B |
| 100k | 69.3 MiB | 727.0 B |

Kernel counters show only coarse RaBitQ NEON rows. There are no TQ/TQ-QJL block-kernel rows for `turboquant2`, so the current TQ2 stage-2 scorer is scalar fallback rather than the full TQ4 SIMD scorer.

## Decision

Do not promote this variant. It answers the middle-ground payload question: naive TQ2 is not the right direction. It is larger and slower than TQ-binary at 100k while also recalling worse, and neither is close to the matched-recall bar.

Task 124 remains open. The next useful work should stay on TQ4 quality and target overhead around payload placement, scoring/materialization fusion, or final exact width strategy rather than reducing TQ payload bits.
