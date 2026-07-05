# Review Request: Task 75 Candidate Funnel Diagnostics

## Scope

Task 75 starts from clean merged `main` after Task 73/74 and adds the missing local measurement surface needed to profile the Intel routing envelope:

- SQL function `ec_spire_index_scan_leaf_candidate_snapshot(index_oid, query)` exposes per-leaf candidate and retention counters for a single SPIRE index scan.
- `ecaz bench spire-pipeline --funnel-output <path>` writes one JSONL candidate-funnel record per measured query.
- `ecaz bench suite` supports `funnel_output` for `spire-pipeline` steps so the Task 75 matrix stays inside the canonical suite runner.

Code commit under review: `4f6de38964403a415a9a5b26cd0d71ec305914bb` (`Add SPIRE candidate funnel diagnostics`).

## Validation

Local validation:

```bash
cargo test -p ecaz-cli spire_pipeline --no-default-features
cargo build -p ecaz-cli --no-default-features
```

Both passed. `cargo build` emitted the existing `LoadedDistributedPlacementConfig.path` dead-code warning.

PG18 install and suite validation are captured under `benchmarks/task75-intel-local-routing-envelope/`:

- `artifacts/install-ecaz-pg18.log`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-run-rerun-port28818.log`
- `artifacts/suite-report.md`

The first actual suite run, `artifacts/suite-run.log`, failed before measurement because it omitted `--port 28818`; the rerun with explicit port completed all 15 steps.

## Local Intel Results

Benchmark packet: `benchmarks/task75-intel-local-routing-envelope/manifest.md`

SPIRE 100k, 200 queries, `boundary_replica_count=0`, `rerank_width=25`:

| Point | nprobe | recall@10 | p50 | p95 | leaf routes | candidates | retained | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tg16 b0 | 16 | 0.8525 | 26.814 ms | 33.414 ms | 2,666 | 2,087,914 | 5,000 | 2,000 |
| tg32 b0 | 32 | 0.9310 | 48.199 ms | 54.407 ms | 3,533 | 2,769,013 | 5,000 | 2,000 |
| tg64 b0 | 64 | 0.9825 | 90.643 ms | 100.316 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |
| tg96 b0 | 96 | 0.9975 | 131.292 ms | 143.238 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |
| tg128 b0 | 96 | 0.9975 | 134.271 ms | 145.134 ms | 3,556 | 2,784,952 | 5,000 | 2,000 |

IVF control, 100k, `nprobe=96`, `rerank_width=500`:

| Point | recall@10 | mean q-time | p50 | p95 | estimated candidates | observed postings visited | rerank rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| IVF nprobe96 | 0.9980 | 37.85 ms | 37.0 ms | 42.0 ms | 75,000 | 77,760 | 500 |

AWS remained off during local work:

- `artifacts/aws-status-1m-after-local-run.log`: profile `1m` paused, DB instance stopped, `$0/hr`.
- `artifacts/aws-status-10k-medium-after-local-run.log`: profile `10k-medium` down, `$0/hr`.

## Notes For Reviewer

- The SQL diagnostic is intended as a measurement surface, not as a planner/runtime dependency.
- `--funnel-output` is blocked with `--production-read-only` because the snapshot reruns local candidate inspection for each query.
- The current local result points at candidate fan-in/routing as the Phase 2 P0 target: SPIRE high recall is comparable to IVF recall but about 3.5x slower at p50 on the Intel desktop.
