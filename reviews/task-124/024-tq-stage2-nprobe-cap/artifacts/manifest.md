# Task 124 Packet 024 Artifact Manifest

- head SHA: `8c0246aec1af884659be4a397475c3c60f8a4833`
- task bucket: `reviews/task-124/024-tq-stage2-nprobe-cap`
- timestamp: `2026-06-30T03:22:20Z`
- lane: local PG18, `tqvector_bench`, host `/Users/peter/.pgrx`, port `28818`
- fixture: staged current real corpus at 10k / 50k / 100k
- quant/index: `ec_ivf`, coarse RaBitQ 1-bit, index-side TurboQuant rerank
- storage format: `coarse_rerank`
- rerank mode: `rerank=heap_f32`, `rerank_placement=index`, `rerank_format=turboquant`, `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- isolation: one fresh index per table/prefix

## Code Change

Commit `8c0246aec1af884659be4a397475c3c60f8a4833` adds an opt-in session GUC,
`ec_ivf.tq_stage2_nprobe_cap`, that caps the effective `ec_ivf.nprobe` only for
persisted index-side TurboQuant stage-2 scans with final exact rerank enabled.
The default `-1` leaves existing behavior unchanged.

## Validation Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| local terminal output | `cargo fmt --check` | passed |
| local terminal output | `cargo test -p ecaz am::ec_ivf::scan::tests::tq_stage2_nprobe_cap --lib --no-default-features --features pg18` | passed, 2 tests |
| local terminal output | `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| local terminal output | `cargo build --release -p ecaz` | passed |
| local terminal output | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` | passed |
| `artifacts/suite-audit.log` | `target/release/ecaz --log-file reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-audit.log bench suite audit --config reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/task124-tq-stage2-nprobe-cap-10-50-100-suite.json` | passed, 18 steps |
| `artifacts/suite-run.log` | `target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-run.log bench suite run --config reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/task124-tq-stage2-nprobe-cap-10-50-100-suite.json --manifest-output reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-manifest.json --results-output reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/results.jsonl` | completed, 18 succeeded / 0 failed |
| `artifacts/suite-status.log` | `target/release/ecaz --log-file reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-status.log bench suite status --manifest reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-manifest.json` | completed, 18 succeeded / 0 failed |
| `artifacts/suite-report.log` | `target/release/ecaz --log-file reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-report.log bench suite report --manifest reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/suite-manifest.json --results-output reviews/task-124/024-tq-stage2-nprobe-cap/artifacts/report-results.jsonl` | report generated |

## A/B Results

| Scale | Variant | Recall@10 | Recall mean q-time | Latency mean | p50 | p95 | p99 | RaBitQ coarse candidates | TQ candidates | TQ scalar candidates | TQ ISA |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 10k | cap off | 1.0000 | 1.36 ms | 1.18 ms | 1.14 ms | 1.32 ms | 1.52 ms | 1,000,000 | 7,500 | 0 | neon |
| 10k | cap 60 | 1.0000 | 1.13 ms | 1.12 ms | 1.09 ms | 1.25 ms | 1.37 ms | 936,366 | 7,500 | 0 | neon |
| 50k | cap off | 0.9980 | 5.03 ms | 4.67 ms | 4.62 ms | 4.80 ms | 4.85 ms | 5,000,000 | 7,500 | 0 | neon |
| 50k | cap 60 | 0.9980 | 4.34 ms | 4.62 ms | 4.56 ms | 4.90 ms | 5.50 ms | 4,525,933 | 7,500 | 0 | neon |
| 100k | cap off | 1.0000 | 9.57 ms | 9.02 ms | 8.95 ms | 9.22 ms | 9.40 ms | 10,000,000 | 7,500 | 0 | neon |
| 100k | cap 60 | 1.0000 | 8.67 ms | 8.63 ms | 8.59 ms | 8.85 ms | 9.03 ms | 9,556,278 | 7,500 | 0 | neon |

## Storage Results

| Scale | ec_ivf index size | ec_ivf index bytes/row |
| --- | ---: | ---: |
| 10k | 10.9 MiB | 1143.6 B |
| 50k | 50.9 MiB | 1066.8 B |
| 100k | 100.8 MiB | 1057.2 B |

## Interpretation

The code path is correctly scoped and the cap applies: coarse RaBitQ candidates
drop at every scale while TQ scorer counters remain fully NEON/SIMD with
`scalar_candidates=0`. Recall is unchanged in this fixture.

Latency is mixed. The cap improves 10k and 100k latency, and slightly improves
50k mean/p50, but 50k p95/p99 regress. This packet therefore supports keeping
the change as an opt-in tuning control, but it is not a Task 124 closeout by
itself.
