# Task 87 Phase 7 Aggregate Matrix

Head SHA: `cbb4d388bdbac0261b446b54d8ef242da130a302f`

Packet path: `reviews/task-87/023-phase7-methodology-and-closeout/`

This matrix supersedes packet 015's Phase 6 aggregate matrix and aggregates
Phase 7 evidence from:

- real10k: `reviews/task-87/021-spire-leaf-lut32-batching/`
- real50k and real100k: `reviews/task-87/022-phase7-50k-100k-counter-suite/`
- HNSW metadata investigation:
  `reviews/task-87/023-phase7-methodology-and-closeout/artifacts/hnsw-reloptions-list.log`

Latency delta is `(on - off) / off`; negative is faster.

## Routed TurboQuant Matrix

| Corpus | AM | Surface note | Recall off | Recall on | p50 off | p50 on | p50 delta | p95 off | p95 on | p95 delta | p99 off | p99 on | p99 delta | On-path Task 87 counters | Storage |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| real10k | IVF | TurboQuant no-QJL LUT32 route | 1.0000 | 1.0000 | 19.3 ms | 16.7 ms | -13.5% | 20.9 ms | 18.0 ms | -13.9% | 22.8 ms | 21.1 ms | -7.5% | `surface=ivf flushes=8000 candidates=2000000 elapsed_ms=2294.054086 lut32_flushes=7800 lut32_candidates=1996800` | total 168.2 MiB, indexes 9.4 MiB |
| real10k | SPIRE | TurboQuant no-QJL LUT32 route | 1.0000 | 1.0000 | 17.686 ms | 15.413 ms | -12.9% | 20.579 ms | 17.951 ms | -12.8% | 22.502 ms | 22.600 ms | +0.4% | `surface=spire flushes=4800 candidates=1551640 elapsed_ms=1793.231978 lut32_flushes=4800 lut32_candidates=1551640` | total 167.0 MiB, indexes 8.2 MiB |
| real50k | IVF | RaBitQ surface; not Task 87 TurboQuant LUT32 | 0.9300 | 0.9300 | 12.2 ms | 12.3 ms | +0.8% | 13.8 ms | 15.5 ms | +12.3% | 15.3 ms | 18.0 ms | +17.6% | zero; not a TurboQuant LUT32 route | total 840.9 MiB, indexes 47.1 MiB |
| real50k | SPIRE | TurboQuant no-QJL LUT32 route | 0.9690 | 0.9690 | 21.997 ms | 18.751 ms | -14.8% | 25.240 ms | 21.833 ms | -13.5% | 27.240 ms | 23.164 ms | -15.0% | `surface=spire flushes=4800 candidates=1739476 elapsed_ms=2006.536739 lut32_flushes=4800 lut32_candidates=1739476` | total 834.3 MiB, indexes 40.5 MiB |
| real100k | IVF | TurboQuant no-QJL LUT32 route | 1.0000 | 1.0000 | 172.7 ms | 146.2 ms | -15.3% | 183.2 ms | 168.0 ms | -8.3% | 186.5 ms | 179.2 ms | -3.9% | `surface=ivf flushes=78200 candidates=20000000 elapsed_ms=23574.111606 lut32_flushes=78200 lut32_candidates=20000000` | total 1.6 GiB, indexes 89.5 MiB |
| real100k | SPIRE | TurboQuant no-QJL LUT32 route | 0.9100 | 0.9100 | 41.179 ms | 35.062 ms | -14.9% | 48.845 ms | 40.653 ms | -16.8% | 51.872 ms | 46.962 ms | -9.5% | `surface=spire flushes=4800 candidates=3842410 elapsed_ms=4486.740935 lut32_flushes=4800 lut32_candidates=3842410` | total 1.6 GiB, indexes 81.8 MiB |

## HNSW Phase 7 Stop Condition

HNSW was probed with candidate-batch scoring enabled on the current real10k,
real50k, and real100k benchmark profiles:

| Corpus | Profile | p50 | p95 | p99 | Task 87 counters | Outcome |
| --- | --- | ---: | ---: | ---: | --- | --- |
| real10k | `task87_phase6_real10k_hnsw` | 4.59 ms | 6.44 ms | 7.41 ms | zero | not a TurboQuant FullLut LUT32 route |
| real50k | `current_intel_real50k_hnsw` | 5.73 ms | 23.7 ms | 34.1 ms | zero | not a TurboQuant FullLut LUT32 route |
| real100k | `current_intel_real100k_hnsw` | 7.58 ms | 43.2 ms | 72.8 ms | zero | not a TurboQuant FullLut LUT32 route |

The reloptions investigation shows the matching real-corpus HNSW indexes are
source-backed profiles, not `storage_format=turboquant` profiles. HNSW
therefore remains on the accepted packet 006 structural route for Task 87,
with any TurboQuant FullLut HNSW real-corpus surface left to Task 91 parity
work.

## Gate Call

Phase 7 lands the 32-candidate LUT kernel under `src/quant/` and routes the
measured SPIRE and IVF TurboQuant no-QJL 4-bit cells through it via
`CandidateBatch`.

- Recall is preserved in all routed off/on cells.
- Real10k IVF, real100k IVF, real50k SPIRE, and real100k SPIRE improve p50,
  p95, and p99 end-to-end latency.
- Real10k SPIRE improves p50 and p95; p99 is effectively flat at +0.4%.
- The real50k IVF row is explicitly not a Task 87 TurboQuant LUT32 route; it is
  the previously documented RaBitQ surface.
- On-path counters show the routed TurboQuant cells reach the LUT32 kernel.
- This closeout does not claim direct scalar-off-path-vs-LUT32-on-path
  scoring-share factors. It reports the direct on-path counter scope and
  end-to-end deltas separately, per packet 021 feedback.
- DiskANN remains closed by packet 005 Stop Condition and packet 009 Task 91
  handoff.
