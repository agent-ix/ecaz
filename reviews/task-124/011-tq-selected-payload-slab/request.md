# Task 124 Packet 011: TQ Selected Payload Slab

This is a TurboQuant-focused optimization checkpoint. It is not Task 124 closeout.

## Summary

I changed the index-side rerank group loader to materialize selected compact payloads into one contiguous slab per loaded group instead of allocating one `Vec<u8>` per selected heap TID.

This targets the Task 124 TQ bottleneck identified by packets 009 and 010: full TQ4 scoring is already SIMD, so the remaining latency work is payload materialization/locality around 768-byte TQ sidecars.

The change is intentionally behavior-preserving:

- selected payload lookup still keys by heap TID
- selected payload bytes remain copied only for requested candidates
- full-chain fallback keeps the old full-group payload path
- TQ scorer counters remain full NEON/SIMD with zero scalar candidates

## Code Commit

- `0af6745d9dbae3120383cbc125d02c136bf41f4b` - `Use slab for selected TQ rerank payloads`

Touched file:

- `src/am/ec_ivf/scan.rs`

## Validation

Passed:

```text
cargo fmt --check
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

The focused scan test command ran 30 tests. The new test `rerank_group_payload_lookup_uses_selected_payload_slab` covers the selected-slab lookup path.

## Benchmark Evidence

Suite config:

- `reviews/task-124/011-tq-selected-payload-slab/artifacts/task124-tq-selected-payload-slab-100k-suite.json`

Suite command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/011-tq-selected-payload-slab/artifacts/task124-tq-selected-payload-slab-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/011-tq-selected-payload-slab/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/011-tq-selected-payload-slab/artifacts/slab-100k/suite-manifest.json
```

Report summary: `completed 4`, `failed 0`, `skipped 0`.

Artifacts:

- `reviews/task-124/011-tq-selected-payload-slab/artifacts/manifest.md`
- `reviews/task-124/011-tq-selected-payload-slab/artifacts/suite-run.log`
- `reviews/task-124/011-tq-selected-payload-slab/artifacts/slab-100k/suite-manifest.json`
- `reviews/task-124/011-tq-selected-payload-slab/artifacts/slab-100k/results.jsonl`
- per-step load, recall, latency, and storage logs under `slab-100k/`

Truth cache files are intentionally untracked.

## Results

Configuration:

- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`
- 100k staged real corpus

Recall at k=10:

| nprobe | packet 010 baseline | packet 011 slab |
| ---: | ---: | ---: |
| 32 | 0.9730 | 0.9730 |
| 64 | 1.0000 | 1.0000 |

Latency:

| nprobe | packet 010 baseline p50/p95/p99 | packet 011 slab p50/p95/p99 |
| ---: | ---: | ---: |
| 32 | 4.90 / 5.47 / 5.70 ms | 4.83 / 5.35 / 5.55 ms |
| 64 | 9.12 / 9.48 / 9.79 ms | 8.91 / 9.14 / 9.25 ms |

Storage:

| packet | ec_ivf index size | per row |
| --- | ---: | ---: |
| 010 baseline | 100.8 MiB | 1057.2 B |
| 011 slab | 100.8 MiB | 1057.2 B |

TQ scorer counters:

| nprobe | quant | isa | scalar_candidates | TQ candidates |
| ---: | --- | --- | ---: | ---: |
| 32 | turboquant | neon | 0 | 7500 |
| 64 | turboquant | neon | 0 | 7500 |

## Decision

Keep the code change. It is a small, measured latency improvement in the specific TQ materialization path Task 124 identified, with no recall or storage regression in the 100k decision run.

This still does not complete Task 124. The best current TQ4 configuration remains far above the f32/source storage baseline: `100.8 MiB` versus `22.5 MiB` at 100k. A full 10k/50k/100k promotion matrix is not justified for closeout until a larger storage/locality change lands.

Next TQ work should target the layout format itself: reduce TQ sidecar bytes persisted or touched, add more direct payload addressing within segments, or fuse stage-2 score/top-k/materialization so fewer 768-byte payloads are copied and scored.
