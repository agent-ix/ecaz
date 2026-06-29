# Task 124 Packet 007 Artifact Manifest

- Task bucket: `reviews/task-124/`
- Packet path: `reviews/task-124/007-tq-binary-stage2-suite/`
- Head SHA: `f1095da153a9e7c35eb90dd2c4e554906bc73577`
- Timestamp: `2026-06-29T03:26:51Z`
- Lane: local PG18, release extension install, `ec_ivf` staged real corpora
- Storage format: `coarse_rerank`
- Coarse frontier: `coarse_format=rabitq`, `coarse_bits=1`, `nlists=64`, recall/latency sweeps at `nprobe=32,64`
- Stage-2 variant: `rerank_placement=index`, `rerank_format=turboquant_binary`, `rerank_width=100`, `stage2_final_rerank_width=15`
- Fixture isolation: one index/table prefix per scale: `task124_tqbin_10k`, `task124_tqbin_50k`, `task124_tqbin_100k`
- Outcome: not promotable and not Task 124 closeout; compact TQ-derived binary sidecar improves bytes and stays full SIMD, but recall is far below matched-recall requirements.

## Code Under Test

Commit `f1095da153a9e7c35eb90dd2c4e554906bc73577` adds an experimental `turboquant_binary` rerank format:

- `src/am/ec_ivf/options.rs`: reloption parse/validation for `turboquant_binary` and `tq_binary`.
- `src/am/ec_ivf/rerank.rs`: TQ-derived no-QJL 4-bit binary sign sidecar codec and batch scorer.
- `src/am/ec_ivf/scan.rs`: permits stage-2 final exact rerank after this TQ-derived stage-2 format.

The 1536D payload is 24 little-endian `u64` sign words, 192 bytes per row. Scoring uses the existing candidate-batch Hamming-word kernel.

## Validation Commands

All validation below passed before this packet was written:

```text
cargo test -p ecaz am::ec_ivf::options --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::rerank --lib --no-default-features --features pg18
cargo fmt --check
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

## Suite Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/007-tq-binary-stage2-suite/artifacts/task124-tq-binary-final15-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/007-tq-binary-stage2-suite/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/007-tq-binary-stage2-suite/artifacts/tq-binary-final15-suite/suite-manifest.json
```

Report summary: `completed 12`, `failed 0`.

## Committed Artifacts

- `task124-tq-binary-final15-suite.json`: SuiteConfig for the 10k/50k/100k TQ-binary matrix.
- `suite-run.log`: suite runner log.
- `tq-binary-final15-suite/suite-manifest.json`: structured suite manifest.
- `tq-binary-final15-suite/results.jsonl`: structured result records.
- `tq-binary-final15-suite/load-*.log`: per-scale load logs.
- `tq-binary-final15-suite/recall-*.log`: per-scale recall logs.
- `tq-binary-final15-suite/latency-*.log`: per-scale latency logs with candidate batch counters.
- `tq-binary-final15-suite/storage-*.log`: per-scale storage logs.

Regenerable `truth-*.json` caches were intentionally not committed.

## Key Results

### Recall at k=10

| Scale | nprobe=32 | nprobe=64 |
| --- | ---: | ---: |
| 10k | 0.9530 | 0.9530 |
| 50k | 0.7920 | 0.7930 |
| 100k | 0.7810 | 0.7980 |

### Latency

| Scale | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.69 ms | 0.75 ms | 0.93 ms |
| 10k | 64 | 1.19 ms | 1.25 ms | 1.37 ms |
| 50k | 32 | 2.32 ms | 2.51 ms | 2.60 ms |
| 50k | 64 | 4.64 ms | 4.84 ms | 5.09 ms |
| 100k | 32 | 4.86 ms | 5.40 ms | 5.69 ms |
| 100k | 64 | 9.08 ms | 9.28 ms | 9.55 ms |

### Storage

| Scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 5.4 MiB | 561.2 B |
| 50k | 23.5 MiB | 491.8 B |
| 100k | 46.1 MiB | 483.0 B |

### Stage-2 Kernel Counters

All latency runs reported the TQ-binary batch scorer on NEON with no scalar candidates:

| Scale | nprobe | isa | scalar_candidates | width_ge32 |
| --- | ---: | --- | ---: | ---: |
| 10k | 32 | neon | 0 | 100 |
| 10k | 64 | neon | 0 | 100 |
| 50k | 32 | neon | 0 | 100 |
| 50k | 64 | neon | 0 | 100 |
| 100k | 32 | neon | 0 | 100 |
| 100k | 64 | neon | 0 | 100 |

## Comparison Against Packet 006

Packet 006 established the current full-TQ final15 slice:

- full TQ no-QJL payload at 1536D: 768 bytes after gamma elision
- 100k full TQ storage: about 100.8 MiB
- 100k f32/source baseline storage: about 22.5 MiB
- full TQ maintained much higher recall than TQ-binary but was not latency/storage-promotable

This packet shows that TQ-binary cuts full-TQ storage materially (`46.1 MiB` at 100k versus about `100.8 MiB`) and scores fully through the SIMD candidate-batch path, but it is still about 2x the f32/source baseline storage and loses too much recall to satisfy Task 124.

## Decision

Do not promote `turboquant_binary` as-is. This is useful negative evidence for the TQ optimization search space:

- The TQ scorer path is full SIMD for this batch shape.
- A 1-bit TQ-derived payload is compact and fast enough to be interesting.
- The quality loss is too large at 50k/100k, so this path does not solve Task 124.

Next TQ exploration should target a middle ground that keeps more TQ information than a binary sign projection while avoiding the full 4-bit payload cost, or should reduce overhead around full TQ without dropping to 1-bit stage-2 scoring.
