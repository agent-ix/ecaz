# Task 121 Phase 2 Local 100k f8 Partial Checkpoint

Head SHA: 96c751d91457499f463a0f17657e018fe17656fe

Scope:

- Local single-PostgreSQL run on `tqvector_bench_task121`; this is not local multi-node evidence and not AWS evidence.
- RaBitQ / TurboQuant focus only.
- Suite-driven run using `suite-phase2-local-100k-f8-boundary-training-run.json`.
- Four 100k f8 cells were loaded and storage-measured.
- One full baseline pipeline completed: `b0_tr10_f8`.
- The next pipeline, `b0_tr50_f8`, was intentionally stopped after baseline completion to avoid spending another long sweep before packaging a usable checkpoint.

Storage:

| cell | index size | index bytes/row | table total |
| --- | ---: | ---: | ---: |
| b0_tr10_f8 | 79.7 MiB | 835.8 B | 1.6 GiB |
| b0_tr50_f8 | 79.6 MiB | 835.2 B | 1.6 GiB |
| b1_tr10_f8 | 157.9 MiB | 1655.2 B | 1.7 GiB |
| b1_tr50_f8 | 157.8 MiB | 1654.5 B | 1.7 GiB |

Completed baseline pipeline: `b0_tr10_f8`

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

Run status:

- `precheck-host`: succeeded
- all four load/storage cells: succeeded
- `truth-cache-100k-q200-k10`: succeeded
- `pipeline-100k_b0_tr10_f8`: succeeded
- `pipeline-100k_b0_tr50_f8`: failed with exit code 1 because the backend was intentionally terminated after baseline completion
- `pipeline-100k_b1_tr10_f8` and `pipeline-100k_b1_tr50_f8`: pending

Interpretation:

- 100k is materially harder than 50k for the b0/tr10/f8 baseline: recall@10 is only 0.9310 at nprobe 32 and needs nprobe 96 to reach 0.9975.
- b1 at 100k again costs roughly 2x index storage versus b0 before any recall comparison.
- This packet does not answer the 100k training/boundary recall interaction; it records the cost of reaching a clean baseline and provides a checkpoint for deciding whether to continue the remaining long 100k sweeps.
