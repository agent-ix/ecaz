# Task 124 Packet 026 Artifact Manifest

- head SHA: `8f7ec6b5bb06c389aeadfdb0e127af9baedbb1db`
- task bucket: `reviews/task-124/026-f32-vs-tq-nprobe60-discriminator`
- timestamp: `2026-06-30T05:35:00Z`
- lane: local PG18, `tqvector_bench`, host `/Users/peter/.pgrx`, port `28818`
- fixture: staged current real corpus at 10k / 50k / 100k
- runner: `ecaz bench suite`
- quant/index: `ec_ivf`, coarse RaBitQ 1-bit
- isolation: one fresh index per table/prefix for each scale/variant
- purpose: answer packet 023 reviewer discriminator for `f32@60` vs `TQ@60`

## Variants

f32/source:

- `rerank=heap_f32`
- `rerank_placement=source`
- `rerank_format=f32`
- `rerank_width=100`
- runtime `ec_ivf.stage2_final_rerank_width=0`

TQ final15:

- `rerank=heap_f32`
- `rerank_placement=index`
- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`
- runtime `ec_ivf.stage2_final_rerank_width=15`

Both:

- `nlists=64`
- `nprobe=60`
- `training_sample_rows=10000`
- `coarse_format=rabitq`
- `coarse_bits=1`
- `storage_format=coarse_rerank`

## Setup Validation

| Artifact | Command | Result |
| --- | --- | --- |
| local terminal output | `cargo build --release -p ecaz` | passed |
| local terminal output | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` | passed |
| `artifacts/suite-audit-r2.log` | `target/release/ecaz --log-file reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-audit-r2.log bench suite audit --config reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/task124-f32-vs-tq-nprobe60-10-50-100-suite.json` | passed, 24 steps |
| `artifacts/suite-run-r2.log` | `target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-run-r2.log bench suite run --config reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/task124-f32-vs-tq-nprobe60-10-50-100-suite.json --manifest-output reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-manifest-r2.json --results-output reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/results-r2.jsonl` | completed, 24 succeeded / 0 failed |
| `artifacts/suite-status-r2.log` | `target/release/ecaz --log-file reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-status-r2.log bench suite status --manifest reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-manifest-r2.json` | completed, 24 succeeded / 0 failed |
| `artifacts/suite-report-r2.log` | `target/release/ecaz --log-file reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-report-r2.log bench suite report --manifest reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/suite-manifest-r2.json --results-output reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/report-results-r2.jsonl` | report generated |

## Results

| Scale | Variant | Recall@10 | NDCG@10 | Recall mean q-time | Latency mean | p50 | p95 | p99 | ec_ivf index | bytes/row |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 1.0000 | 1.0000 | 1.27 ms | 1.23 ms | 1.22 ms | 1.38 ms | 1.43 ms | 2.9 MiB | 305.6 B |
| 10k | TQ final15 | 1.0000 | 1.0000 | 1.17 ms | 1.15 ms | 1.13 ms | 1.28 ms | 1.37 ms | 10.9 MiB | 1143.6 B |
| 50k | f32/source | 1.0000 | 1.0000 | 5.01 ms | 4.57 ms | 4.48 ms | 5.32 ms | 5.83 ms | 11.6 MiB | 243.3 B |
| 50k | TQ final15 | 0.9980 | 1.0000 | 4.34 ms | 4.25 ms | 4.23 ms | 4.47 ms | 4.54 ms | 50.9 MiB | 1066.8 B |
| 100k | f32/source | 1.0000 | 1.0000 | 9.78 ms | 9.49 ms | 9.46 ms | 9.76 ms | 9.92 ms | 22.5 MiB | 235.8 B |
| 100k | TQ final15 | 1.0000 | 1.0000 | 8.90 ms | 8.83 ms | 8.77 ms | 9.01 ms | 9.22 ms | 100.8 MiB | 1057.2 B |

## Counters

| Scale | Variant | Coarse candidates | TQ candidates | TQ scalar candidates | TQ elapsed | ISA |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 10k | f32/source | 936,366 | 0 | n/a | n/a | n/a |
| 10k | TQ final15 | 936,366 | 7,500 | 0 | 1.811008 ms | neon |
| 50k | f32/source | 4,525,933 | 0 | n/a | n/a | n/a |
| 50k | TQ final15 | 4,525,933 | 7,500 | 0 | 1.851708 ms | neon |
| 100k | f32/source | 9,556,278 | 0 | n/a | n/a | n/a |
| 100k | TQ final15 | 9,556,278 | 7,500 | 0 | 1.907458 ms | neon |

## Interpretation

The reviewer discriminator is negative. f32/source at `nprobe=60` preserves
recall at 10k, 50k, and 100k. Therefore `nprobe=60` is not a TQ-specific
frontier advantage under the stated test: TQ does not uniquely allow the
shallower frontier while f32 fails.

TQ final15 is still faster in this specific run, but it is not recall-superior:
at 50k it reports `0.9980` recall while f32/source reports `1.0000`. It also
retains the established storage gap: `100.8 MiB` vs `22.5 MiB` at 100k.

Do not use the nprobe cap or nprobe60 matrix as Task 124 closeout evidence for
TQ-attributable speed. It remains an operating-point knob.
