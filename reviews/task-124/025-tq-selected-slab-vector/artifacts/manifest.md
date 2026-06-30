# Task 124 Packet 025 Artifact Manifest

- head SHA: `c4f850d3d9664321e5dcb92f444ce1351b0d7929`
- task bucket: `reviews/task-124/025-tq-selected-slab-vector`
- timestamp: `2026-06-30T04:20:00Z`
- lane: local PG18, `tqvector_bench`, host `/Users/peter/.pgrx`, port `28818`
- fixture: staged current real corpus at 10k / 50k / 100k
- quant/index: `ec_ivf`, coarse RaBitQ 1-bit, index-side TurboQuant rerank
- storage format: `coarse_rerank`
- rerank mode: `rerank=heap_f32`, `rerank_placement=index`, `rerank_format=turboquant`, `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- isolation: one fresh index per table/prefix
- code status: temporary uncommitted experiment measured and then reverted; no
  code change proposed for landing

## Experiment

The temporary diff in `artifacts/discarded-selected-slab-vector.diff` replaced
selected rerank payload slab heap-TID lookup hash maps with compact vectors and
linear lookup. The goal was to reduce tiny allocation/hash overhead in the TQ
selected payload path.

The measured result was mixed to negative, so the code was reverted. This packet
is evidence only.

## Validation Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| local terminal output | `cargo fmt --check` | passed after reverting the experiment |
| `artifacts/suite-audit.log` | `target/release/ecaz --log-file reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-audit.log bench suite audit --config reviews/task-124/025-tq-selected-slab-vector/artifacts/task124-tq-selected-slab-vector-10-50-100-suite.json` | passed, 18 steps |
| `artifacts/suite-run.log` | `target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-run.log bench suite run --config reviews/task-124/025-tq-selected-slab-vector/artifacts/task124-tq-selected-slab-vector-10-50-100-suite.json --manifest-output reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-manifest.json --results-output reviews/task-124/025-tq-selected-slab-vector/artifacts/results.jsonl` | completed, 18 succeeded / 0 failed |
| `artifacts/suite-status.log` | `target/release/ecaz --log-file reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-status.log bench suite status --manifest reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-manifest.json` | completed, 18 succeeded / 0 failed |
| `artifacts/suite-report.log` | `target/release/ecaz --log-file reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-report.log bench suite report --manifest reviews/task-124/025-tq-selected-slab-vector/artifacts/suite-manifest.json --results-output reviews/task-124/025-tq-selected-slab-vector/artifacts/report-results.jsonl` | report generated |

## A/B Results

| Scale | Variant | Recall@10 | Recall mean q-time | Latency mean | p50 | p95 | p99 | RaBitQ coarse candidates | TQ candidates | TQ scalar candidates | TQ elapsed | TQ ISA |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 10k | cap off | 1.0000 | 1.31 ms | 1.21 ms | 1.17 ms | 1.36 ms | 1.66 ms | 1,000,000 | 7,500 | 0 | 1.858348 ms | neon |
| 10k | cap 60 | 1.0000 | 1.17 ms | 1.12 ms | 1.09 ms | 1.21 ms | 1.34 ms | 936,366 | 7,500 | 0 | 1.797417 ms | neon |
| 50k | cap off | 0.9980 | 5.29 ms | 4.91 ms | 4.81 ms | 5.35 ms | 6.11 ms | 5,000,000 | 7,500 | 0 | 1.892330 ms | neon |
| 50k | cap 60 | 0.9980 | 4.61 ms | 4.43 ms | 4.39 ms | 4.68 ms | 4.83 ms | 4,525,933 | 7,500 | 0 | 1.916914 ms | neon |
| 100k | cap off | 1.0000 | 9.83 ms | 9.41 ms | 9.34 ms | 9.60 ms | 9.73 ms | 10,000,000 | 7,500 | 0 | 1.894086 ms | neon |
| 100k | cap 60 | 1.0000 | 9.13 ms | 9.16 ms | 8.99 ms | 10.1 ms | 12.0 ms | 9,556,278 | 7,500 | 0 | 1.979873 ms | neon |

## Storage Results

| Scale | ec_ivf index size | ec_ivf index bytes/row |
| --- | ---: | ---: |
| 10k | 10.9 MiB | 1143.6 B |
| 50k | 50.9 MiB | 1066.8 B |
| 100k | 100.8 MiB | 1057.2 B |

## Comparison To Packet 024

| Scale | Variant | Packet 024 p50/p95/p99 | This packet p50/p95/p99 | Decision |
| --- | --- | ---: | ---: | --- |
| 10k | cap off | 1.14 / 1.32 / 1.52 ms | 1.17 / 1.36 / 1.66 ms | worse |
| 10k | cap 60 | 1.09 / 1.25 / 1.37 ms | 1.09 / 1.21 / 1.34 ms | slight tail win |
| 50k | cap off | 4.62 / 4.80 / 4.85 ms | 4.81 / 5.35 / 6.11 ms | worse |
| 50k | cap 60 | 4.56 / 4.90 / 5.50 ms | 4.39 / 4.68 / 4.83 ms | better |
| 100k | cap off | 8.95 / 9.22 / 9.40 ms | 9.34 / 9.60 / 9.73 ms | worse |
| 100k | cap 60 | 8.59 / 8.85 / 9.03 ms | 8.99 / 10.1 / 12.0 ms | worse tail |

## Interpretation

Recall and storage were unchanged, and TQ scoring remained fully NEON/SIMD with
`scalar_candidates=0`. Latency did not improve reliably. The 100k cap60 tail
regression is enough to reject this approach.

This is not a Task 124 closeout and not a code-review request. It documents a
discarded TQ-specific optimization so the next slice can move to a larger
latency lever.
