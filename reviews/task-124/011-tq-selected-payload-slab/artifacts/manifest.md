# Task 124 Packet 011 Artifact Manifest

- Task bucket: `reviews/task-124/`
- Packet path: `reviews/task-124/011-tq-selected-payload-slab/`
- Head SHA: `0af6745d9dbae3120383cbc125d02c136bf41f4b`
- Timestamp: `2026-06-29T08:26:51-0700`
- Lane: local PG18, release extension install, `ec_ivf` staged real 100k corpus
- Storage format: `coarse_rerank`
- Coarse frontier: `coarse_format=rabitq`, `coarse_bits=1`, `nlists=64`, recall/latency sweeps at `nprobe=32,64`
- Stage-2 variant: `rerank_placement=index`, `rerank_format=turboquant`, runtime `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- Fixture isolation: one index/table prefix: `task124_tq_slab_w75_g50_100k`
- Outcome: code change is a modest measured TQ latency win at 100k, with recall and storage unchanged; not Task 124 closeout.

## Code Under Test

Commit `0af6745d9dbae3120383cbc125d02c136bf41f4b` changes `src/am/ec_ivf/scan.rs`:

- Replaces per-selected-payload `Vec<u8>` allocations in `LoadedRerankGroup` with a contiguous selected payload slab.
- Stores heap TID -> slab index metadata instead of heap TID -> payload vec.
- Keeps full-group fallback unchanged for non-direct group loads.
- Adds `rerank_group_payload_lookup_uses_selected_payload_slab` unit coverage.

## Validation Commands

Passed before this packet was written:

```text
cargo fmt --check
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

Focused scan tests: `30 passed; 0 failed`.

## Suite Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/011-tq-selected-payload-slab/artifacts/task124-tq-selected-payload-slab-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/011-tq-selected-payload-slab/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/011-tq-selected-payload-slab/artifacts/slab-100k/suite-manifest.json
```

Report summary: `completed 4`, `failed 0`, `skipped 0`.

## Committed Artifacts

- `task124-tq-selected-payload-slab-100k-suite.json`: SuiteConfig for the post-change 100k TQ run.
- `suite-run.log`: suite runner log.
- `slab-100k/suite-manifest.json`: structured suite manifest.
- `slab-100k/results.jsonl`: structured result records.
- `slab-100k/load-*.log`: load log.
- `slab-100k/recall-*.log`: recall log.
- `slab-100k/latency-*.log`: latency log with candidate-batch counters.
- `slab-100k/storage-*.log`: storage log.

Regenerable `truth-*.json` caches were intentionally not committed.

## Key Results

### Recall at k=10

| nprobe | packet 010 baseline | packet 011 slab |
| ---: | ---: | ---: |
| 32 | 0.9730 | 0.9730 |
| 64 | 1.0000 | 1.0000 |

### Latency

| nprobe | packet 010 baseline p50/p95/p99 | packet 011 slab p50/p95/p99 |
| ---: | ---: | ---: |
| 32 | 4.90 / 5.47 / 5.70 ms | 4.83 / 5.35 / 5.55 ms |
| 64 | 9.12 / 9.48 / 9.79 ms | 8.91 / 9.14 / 9.25 ms |

### Storage

| packet | ec_ivf index size | per row |
| --- | ---: | ---: |
| 010 baseline | 100.8 MiB | 1057.2 B |
| 011 slab | 100.8 MiB | 1057.2 B |

### Kernel Counters

| nprobe | quant | isa | scalar_candidates | TQ candidates |
| ---: | --- | --- | ---: | ---: |
| 32 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

## Decision

Keep the code change as a narrow TQ materialization improvement.

Do not claim Task 124 closeout. The full-TQ4 stage-2 path still has an unacceptable storage gap versus f32/source, and this packet only measured the post-change 100k decision point against the packet 010 baseline.
