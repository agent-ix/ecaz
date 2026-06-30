# Task 124 Packet 037 Artifact Manifest

- head SHA: `0b3fd57f713297b9c07ceadc364ad6a698021a75`
- task bucket: `reviews/task-124/037-tq2-dim768-real-index/`
- packet topic: real-index reduced-dimension TQ2 validation
- timestamp: 2026-06-30
- database / host / port: `tqvector_bench` / `/Users/peter/.pgrx` / `28818`
- runner: `./target/release/ecaz bench suite`
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_*.tsv`
- storage format: `coarse_rerank`
- rerank path: `rerank=heap_f32`, `rerank_placement=index`, `rerank_format=turboquant2_768`, `rerank_width=100`, `stage2_final_rerank_width=15`
- scan sweep: `nprobe=32,64`
- isolated surface: one fresh prefix per scale, `task124_tq2d768_{10k,50k,100k}`

## Commands

- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `./target/release/ecaz bench suite audit --config reviews/task-124/037-tq2-dim768-real-index/artifacts/task124-tq2-dim768-final15-suite.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-audit.log`
- `./target/release/ecaz bench suite run --config reviews/task-124/037-tq2-dim768-real-index/artifacts/task124-tq2-dim768-final15-suite.json --artifact-dir reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-run.log`
- `./target/release/ecaz bench suite status --manifest reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/suite-manifest.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-status.log`
- `./target/release/ecaz bench suite report --manifest reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/suite-manifest.json --log-file reviews/task-124/037-tq2-dim768-real-index/artifacts/suite-report.log`

## Artifacts

- `task124-tq2-dim768-final15-suite.json`: suite config.
- `suite-audit.log`: audit result, 12 steps.
- `suite-run.log`: suite run transcript.
- `suite-status.log`: status result, `completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `suite-report.log`: parsed report.
- `tq2-dim768-final15-suite/suite-manifest.json`: suite manifest.
- `tq2-dim768-final15-suite/results.jsonl`: parsed structured result lines.
- `tq2-dim768-final15-suite/load-*.log`: fresh real-index builds with `rerank_format=turboquant2_768`.
- `tq2-dim768-final15-suite/recall-*.log`: recall and NDCG results.
- `tq2-dim768-final15-suite/latency-*.log`: latency and block-kernel counter results.
- `tq2-dim768-final15-suite/storage-*.log`: storage results.

The suite emitted `truth-*-k10.json` caches under the artifact directory. They are regenerable ground-truth caches and are intentionally not part of the committed packet.

## Key Results

Recall / latency:

| Scale | nprobe | recall@10 | NDCG@10 | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.9530 | 0.9989 | 0.70 ms | 0.81 ms | 0.99 ms |
| 10k | 64 | 0.9530 | 0.9989 | 1.17 ms | 1.25 ms | 1.32 ms |
| 50k | 32 | 0.7700 | 0.9914 | 2.36 ms | 2.58 ms | 2.67 ms |
| 50k | 64 | 0.7710 | 0.9915 | 4.73 ms | 4.91 ms | 5.07 ms |
| 100k | 32 | 0.7620 | 0.9898 | 4.95 ms | 5.45 ms | 5.81 ms |
| 100k | 64 | 0.7710 | 0.9928 | 9.21 ms | 9.40 ms | 9.70 ms |

TQ scorer elapsed inside the real latency path, NEON + scalar tail:

| Scale | nprobe | `turboquant2` packet 036 | `turboquant2_768` packet 037 | Delta |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 4.027461 ms | 2.036876 ms | -49.4% |
| 10k | 64 | 4.183670 ms | 2.062286 ms | -50.7% |
| 50k | 32 | 4.297787 ms | 2.138413 ms | -50.2% |
| 50k | 64 | 4.354213 ms | 2.140495 ms | -50.8% |
| 100k | 32 | 4.444708 ms | 2.220708 ms | -50.0% |
| 100k | 64 | 4.602538 ms | 2.160324 ms | -53.1% |

Packet 037 TQ scorer attribution for every latency run used the SIMD row for 9,600 candidates and the scalar row for 400 candidates:

- 100k/nprobe64 NEON row: `quant=turboquant_qjl isa=neon candidates=9600 elapsed_ms=1.376537`
- 100k/nprobe64 scalar tail row: `quant=turboquant_qjl isa=scalar candidates=400 elapsed_ms=0.783787`

Storage:

| Scale | ec_ivf index size | ec_ivf bytes/row |
| --- | ---: | ---: |
| 10k | 5.4 MiB | 562.0 B |
| 50k | 23.5 MiB | 492.2 B |
| 100k | 46.1 MiB | 483.2 B |

## Outcome

The reduced-dimension production path is real and workload-tested: the suite built fresh IVF indexes with `rerank_format=turboquant2_768` and recorded in-engine TQ scorer counters. The scorer speed improvement versus full-dimension TQ2 is real at the scorer level, about 49-53% lower elapsed time. It is not a usable TQ speedup for the stage-2 contract because recall remains broken at 50k and 100k.
