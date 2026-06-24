# Task 121 Phase 2 local 100k f8 partial checkpoint

This packet records a local-only, single-PostgreSQL 100k f8 checkpoint for the
Task 121 Phase 2 boundary/training follow-up. It is not local multi-node
evidence, not AWS evidence, and not closeout evidence.

What completed:

- Loaded and storage-measured four 100k f8 cells:
  - `b0_tr10_f8`
  - `b0_tr50_f8`
  - `b1_tr10_f8`
  - `b1_tr50_f8`
- Built the 200-query truth cache.
- Completed one full baseline pipeline sweep for `b0_tr10_f8`.

What was intentionally stopped:

- After `pipeline-100k_b0_tr10_f8` completed, the next pipeline
  `pipeline-100k_b0_tr50_f8` was terminated to avoid spending another long sweep
  before packaging a checkpoint. The suite manifest therefore shows that step as
  failed with exit code 1 and the two b1 pipeline steps as pending.

Key results:

| cell | index size | index bytes/row | table total |
| --- | ---: | ---: | ---: |
| b0_tr10_f8 | 79.7 MiB | 835.8 B | 1.6 GiB |
| b0_tr50_f8 | 79.6 MiB | 835.2 B | 1.6 GiB |
| b1_tr10_f8 | 157.9 MiB | 1655.2 B | 1.7 GiB |
| b1_tr50_f8 | 157.8 MiB | 1654.5 B | 1.7 GiB |

Completed 100k baseline `b0_tr10_f8` pipeline:

| nprobe | p50 latency | p95 latency | recall@10 |
| ---: | ---: | ---: | ---: |
| 4 | 125.369 ms | 163.540 ms | 0.5500 |
| 8 | 245.514 ms | 307.142 ms | 0.7250 |
| 12 | 386.604 ms | 459.467 ms | 0.8010 |
| 16 | 518.468 ms | 608.491 ms | 0.8525 |
| 24 | 817.531 ms | 916.031 ms | 0.9045 |
| 32 | 1104.833 ms | 1265.117 ms | 0.9310 |
| 48 | 1715.345 ms | 1914.705 ms | 0.9645 |
| 64 | 2308.951 ms | 2485.750 ms | 0.9825 |
| 96 | 3455.559 ms | 3687.544 ms | 0.9975 |

Evidence:

- Artifact manifest: `artifacts/manifest.md`
- Compact summary: `artifacts/summary-100k-f8-baseline.md`
- Suite config: `artifacts/suite-phase2-local-100k-f8-boundary-training-run.json`
- Suite status: `artifacts/suite-phase2-local-100k-f8-boundary-training-run-manifest.json`
- Completed pipeline log: `artifacts/pipeline-100k_b0_tr10_f8.log`
- Storage logs: `artifacts/storage-100k_*.log`

Reviewer notes requested:

- Treat this as a partial checkpoint only.
- The 100k boundary/training recall interaction is still unanswered because the
  remaining three pipeline sweeps were not completed.
- This packet does confirm the 100k b0/tr10 baseline is much harder than 50k and
  that b1 remains roughly a 2x index-size cost at 100k.
