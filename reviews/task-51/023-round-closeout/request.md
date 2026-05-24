# Review Request: Task 51 Round Closeout

Please review this Task 51 closeout packet. It summarizes the IVF/RaBitQ-only
round, maps exit criteria to durable packets, and records the final AWS state.

## Completion Checklist

Task 51 exit criteria:

| Criterion | Evidence | Status |
| --- | --- | --- |
| 1M counter baseline exists | Packets 001, 007, 011, 017; AWS EXPLAIN in `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/explain-1m-rabitq1-rerank-p256.log` | Done |
| At least two low-risk experiments measured | Exp 2 geometry, Exp 3 scratch SoA, Exp 4 locality gate, Exp 5 adaptive, Exp 7 sidecar | Done |
| Posting Layout v2 decision made | Exp 3 did not meet promotion gate; no Layout v2 work started | Rejected |
| Final packet identifies Pareto points | See Pareto Summary below | Done |
| Remaining work split into follow-ups | See Follow-Ups below | Done |
| AWS final gate complete | Packet 017; suite status `completed=6 failed=0 missing_artifacts=0` | Done |
| AWS spend stopped | `cloud-status-after-down.log`: state `down`, `$0.00/hr` running | Done |

## AWS Final Gate

The current-head AWS final gate ran on host SHA
`902e8e066944d4cabfb26ee5cc9039b466856891`, restored from
`snap-0e0632400184fadd4`.

Final suite status:

```text
[suite:task51-aws-ivf-rabitq-current-head-final-gate] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Final cloud state:

```text
state: down
snapshot: snap-0758119609e81ab7f
cost: ~$0.00/hr running, ~$4.00/mo retained storage
```

## Pareto Summary

Baseline current-head IVF/RaBitQ, q=500/q=200:

| Cell | Recall@10 | p50 | p95 | p99 | Storage |
| --- | ---: | ---: | ---: | ---: | ---: |
| preserved `rabitq`, `heap_f32`, nprobe=256 | 0.9936 | 69.1 ms | 75.7 ms | 80.2 ms | 298 MB ec_ivf index |

Sidecar real-I/O upper-bound cells, q=200, nprobe=128, candidate_k=50:

| Cell | Recall@10 | sidecar p50 | sidecar p95 | sidecar p99 | total-bound p50 | Sidecar size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| f16 random-id c1 | 0.9815 | 18.761 ms | 324.069 ms | 529.692 ms | 63.026 ms | 2.83 GiB |
| f16 TID-sorted c1 | 0.9815 | 0.523 ms | 0.787 ms | 1.920 ms | 43.619 ms | 2.83 GiB |
| rabitq8 random-id c1 | 0.9455 | 1.918 ms | 4.819 ms | 11.585 ms | 45.166 ms | 1.43 GiB |
| rabitq8 TID-sorted c1 | 0.9455 | 0.413 ms | 0.437 ms | 0.535 ms | 43.499 ms | 1.43 GiB |
| rabitq8 TID-sorted c4 | 0.9455 | 1.121 ms | 1.723 ms | 334.866 ms | 41.615 ms | 1.43 GiB |

Best measured Pareto point in this round: `rabitq8` TID-sorted sidecar
measurement gives the smallest measured sidecar footprint and lowest sidecar
p50/p95. Its total-bound p50 is `43.499 ms` versus the current-head baseline
`69.1 ms`, a projected p50 improvement of about 37% at lower measured recall
(`0.9455` vs `0.9936`). This remains measurement-only. It is not a product
storage-format decision.

## Closed Experiments

- Exp 2 geometry: local evidence showed `nlists=128` can win at 50k matched recall, but 100k did not prove a stable 1M promotion. Do not claim it as the final AWS shape from this round.
- Exp 3 scratch SoA: closed for this round. The local x86 evidence is below gate; the Graviton NEON kernel measurement is the only open question and is captured as follow-up #2. This round does not use Exp 3 to justify Layout v2.
- Exp 4 heap locality: rejected by counter arithmetic in packet 021; exact rerank is too small a share to meet the 15% gate.
- Exp 5 adaptive nprobe/rerank width: closed negative in packet 018; no threshold preserved recall while producing the required p50 win.
- Exp 6 Posting Layout v2: rejected for this round because Exp 3 did not produce the required gate-opening evidence.
- Exp 7 sidecar: measured as upper-bound / Pareto evidence only.

## Follow-Ups

1. Product sidecar design task: decide whether `rabitq8` or f16 TID-sorted sidecar should become a product feature. The starting target is the measured `rabitq8` TID-sorted point: about 37% projected p50 improvement (`43.499 ms` total-bound p50 vs `69.1 ms` baseline) at `1.43 GiB` sidecar size. The design must explicitly handle the tail risks shown here: random-id tail blowup (`f16` random-id c1 p99 `529.692 ms`) and TID-sorted concurrency tail blowup (`rabitq8` TID-sorted c4 p99 `334.866 ms`). Diagnose the c4 p99 sidecar tail before any product decision: likely candidates are page-cache cliff under concurrent fetch, buffer-manager interaction, or queue stalls. The harness is not a product read path, and the c4 result shows concurrency is not free. Use packets 016, 020, and 022 as the read-mode and assumption baseline.
2. Optional Graviton scratch SoA microbench: one focused AWS cell only if the team wants a definitive ARM/NEON answer for Exp 3; otherwise leave Layout v2 rejected for this round.
3. AWS workflow cleanup: add a standard `ecaz cloud` artifact retrieval path for on-host suite runs, so future agents do not use direct SSM/S3 glue.
