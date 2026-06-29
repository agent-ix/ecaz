# Task 124 Packet 007: TQ Binary Stage-2 Suite

This is a Task 124 TurboQuant-only optimization checkpoint, not closeout.

## Summary

I added an experimental `turboquant_binary` rerank format to test whether a compact TQ-derived stage-2 sidecar could reduce full-TQ payload cost while keeping the existing final exact f32 safety boundary.

The implementation stores a 1536D TQ-derived binary sign sidecar as 24 `u64` words, 192 bytes per row, and scores it with the existing SIMD Hamming-word candidate-batch kernel. It is deliberately scoped to TQ stage-2 exploration and does not touch SPIRE.

## Code Commit

- `f1095da153a9e7c35eb90dd2c4e554906bc73577` - `Add TQ binary stage2 rerank format`

Touched files:

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/rerank.rs`
- `src/am/ec_ivf/scan.rs`

## Validation

Passed:

```text
cargo test -p ecaz am::ec_ivf::options --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::rerank --lib --no-default-features --features pg18
cargo fmt --check
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

## Benchmark Evidence

Suite config:

- `reviews/task-124/007-tq-binary-stage2-suite/artifacts/task124-tq-binary-final15-suite.json`

Suite command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/007-tq-binary-stage2-suite/artifacts/task124-tq-binary-final15-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/007-tq-binary-stage2-suite/artifacts/suite-run.log
```

Artifacts:

- `reviews/task-124/007-tq-binary-stage2-suite/artifacts/manifest.md`
- `reviews/task-124/007-tq-binary-stage2-suite/artifacts/suite-run.log`
- `reviews/task-124/007-tq-binary-stage2-suite/artifacts/tq-binary-final15-suite/suite-manifest.json`
- `reviews/task-124/007-tq-binary-stage2-suite/artifacts/tq-binary-final15-suite/results.jsonl`

Suite completed 12 steps with 0 failures.

## Results

Recall fails matched-recall requirements:

| Scale | nprobe=32 | nprobe=64 |
| --- | ---: | ---: |
| 10k | 0.9530 | 0.9530 |
| 50k | 0.7920 | 0.7930 |
| 100k | 0.7810 | 0.7980 |

Latency is fast:

| Scale | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.69 ms | 0.75 ms | 0.93 ms |
| 10k | 64 | 1.19 ms | 1.25 ms | 1.37 ms |
| 50k | 32 | 2.32 ms | 2.51 ms | 2.60 ms |
| 50k | 64 | 4.64 ms | 4.84 ms | 5.09 ms |
| 100k | 32 | 4.86 ms | 5.40 ms | 5.69 ms |
| 100k | 64 | 9.08 ms | 9.28 ms | 9.55 ms |

Storage improves versus full TQ but is still not competitive with the f32/source baseline:

| Scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 5.4 MiB | 561.2 B |
| 50k | 23.5 MiB | 491.8 B |
| 100k | 46.1 MiB | 483.0 B |

Candidate-batch counters confirm full SIMD scoring for the TQ-binary stage:

| Scale | nprobe | isa | scalar_candidates | width_ge32 |
| --- | ---: | --- | ---: | ---: |
| 10k | 32 | neon | 0 | 100 |
| 10k | 64 | neon | 0 | 100 |
| 50k | 32 | neon | 0 | 100 |
| 50k | 64 | neon | 0 | 100 |
| 100k | 32 | neon | 0 | 100 |
| 100k | 64 | neon | 0 | 100 |

## Decision

Do not promote this variant. It answers a specific Task 124 question: a 1-bit TQ-derived sidecar can be compact and fully SIMD, but it destroys recall at 50k/100k. This packet narrows the TQ optimization space; it does not complete Task 124.

Recommended next slice: test a middle-ground TQ representation or full-TQ overhead reduction that preserves more information than binary signs while still reducing payload cost or per-candidate materialization overhead.
