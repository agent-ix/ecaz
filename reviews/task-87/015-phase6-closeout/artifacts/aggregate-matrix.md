# Task 87 Phase 6 Aggregate Matrix

Head SHA: `9e4d2ad3642b63eec543d9710ca521d1b8f82787`

Packet path: `reviews/task-87/015-phase6-closeout/`

This matrix aggregates the Phase 6 off/on evidence from:

- real10k: `reviews/task-87/012-phase6-suite-prep/`
- real50k: `reviews/task-87/013-phase6-real50k-matrix/`
- real100k: `reviews/task-87/014-phase6-real100k-matrix/`

The off/on cells use the same installed build and flip only the measurement
GUC for the candidate-batch route:

- HNSW: `ec_hnsw.candidate_batch_scoring=off/on`
- IVF: `ec_ivf.scratch_soa_batch_decode=off/on`
- SPIRE: `ec_spire.candidate_batch_scoring=off/on`

DiskANN is represented by the accepted Stop Condition packet
`reviews/task-87/005-phase4-diskann-stop-condition/` and the Task 91 handoff
approved in `reviews/task-87/009-scope-walk-back-and-task-91-handoff/`.

## Aggregate Results

Latency delta is `(on - off) / off`; negative is faster.

| Corpus | AM | Recall off | Recall on | p50 off | p50 on | p50 delta | p95 off | p95 on | p95 delta | p99 off | p99 on | p99 delta | Storage |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| real10k | HNSW | 0.6550 | 0.6550 | 32.6 ms | 31.6 ms | -3.1% | 44.2 ms | 44.1 ms | -0.2% | 49.0 ms | 47.5 ms | -3.1% | total 171.8 MiB, indexes 13.0 MiB |
| real10k | IVF | 1.0000 | 1.0000 | 119.6 ms | 117.4 ms | -1.8% | 135.9 ms | 128.7 ms | -5.3% | 149.4 ms | 138.5 ms | -7.3% | total 168.2 MiB, indexes 9.4 MiB |
| real10k | SPIRE | 1.0000 | 1.0000 | 168.137 ms | 106.142 ms | -36.9% | 187.051 ms | 122.507 ms | -34.5% | 192.607 ms | 132.642 ms | -31.1% | total 167.0 MiB, indexes 8.2 MiB |
| real50k | HNSW | 0.9180 | 0.9180 | 32.4 ms | 31.3 ms | -3.4% | 42.1 ms | 37.9 ms | -10.0% | 58.5 ms | 41.9 ms | -28.4% | total 860.0 MiB, indexes 66.2 MiB |
| real50k | IVF | 0.9300 | 0.9300 | 264.0 ms | 264.3 ms | +0.1% | 289.7 ms | 292.9 ms | +1.1% | 311.6 ms | 308.5 ms | -1.0% | total 840.9 MiB, indexes 47.1 MiB |
| real50k | SPIRE | 0.9690 | 0.9690 | 224.610 ms | 160.449 ms | -28.6% | 255.674 ms | 180.921 ms | -29.2% | 266.182 ms | 186.580 ms | -29.9% | total 834.3 MiB, indexes 40.5 MiB |
| real100k | HNSW | 0.8980 | 0.8980 | 35.6 ms | 35.5 ms | -0.3% | 43.8 ms | 42.9 ms | -2.1% | 51.6 ms | 50.1 ms | -2.9% | total 1.7 GiB, indexes 132.4 MiB |
| real100k | IVF | 1.0000 | 1.0000 | 1064.2 ms | 960.5 ms | -9.7% | 1114.9 ms | 1018.0 ms | -8.7% | 1131.1 ms | 1048.6 ms | -7.3% | total 1.6 GiB, indexes 89.5 MiB |
| real100k | SPIRE | 0.9100 | 0.9100 | 414.768 ms | 273.031 ms | -34.2% | 471.651 ms | 298.031 ms | -36.8% | 495.905 ms | 308.541 ms | -37.8% | total 1.6 GiB, indexes 81.8 MiB |

## Recall And Storage

Recall is unchanged in every off/on cell in the matrix. Storage is also
unchanged by construction because the off/on comparison flips only a session
route GUC against the same already-built index surface.

## Scoring-Share Attribution

The suite emits different attribution surfaces by AM:

- SPIRE uses `ecaz bench spire-pipeline`, which emits routing budget counters,
  local pipeline counters, and coordinator query metrics. The candidate step is
  the dominant counted candidate surface in all three slices:
  - real10k on: candidates `item_sum=775820`, `ready_sum=2500`,
    `blocked_sum=773320`, `candidate_sum=775820`, heap rerank
    `ready_sum=2500`.
  - real50k on: candidates `item_sum=869738`, `ready_sum=2500`,
    `blocked_sum=867238`, `candidate_sum=869738`, heap rerank
    `ready_sum=2500`.
  - real100k on: candidates `item_sum=1921205`, `ready_sum=2500`,
    `blocked_sum=1918705`, `candidate_sum=1921205`, heap rerank
    `ready_sum=2500`.
- HNSW and IVF suite cells emit recall, query-time, latency, and storage, but
  not an isolated scoring-share counter. Their closeout call therefore uses
  recall-preserving off/on route deltas plus the structural route evidence from
  packets 006 and 011, not a direct scoring-share factor.

## Gate Call

Task 87 shipped and measured candidate-batch routes on the existing per-AM
codec surfaces that remain in scope after the packet 009 Task 91 walk-back.

- SPIRE: recall-preserving in all three corpus sizes with consistent
  end-to-end pipeline gains: p50 improves 28.6% to 36.9%, p95 improves 29.2%
  to 36.8%, and p99 improves 29.9% to 37.8%. Packet 001 blocker B4's
  structural-slice carve-out applies because SPIRE already had a chunked
  scorer and Task 87 did not land a new block kernel.
- IVF: recall-preserving in all three corpus sizes. real10k and real100k
  improve; real50k RaBitQ is flat/slightly worse at p50/p95 and slightly
  faster at p99. Packet 001 blocker B4's structural-slice carve-out applies to
  the batch route reachability work, and this cell remains documented as a
  measurement miss rather than a claimed perf win.
- HNSW: recall-preserving in all three corpus sizes. real50k improves
  materially in p95/p99, real10k is small positive, and real100k p50 is
  effectively flat. This is consistent with the Phase 5 HNSW design note that
  per-frontier graph traversal can expose small batches where traversal
  dominates the end-to-end wall time.
- DiskANN: Task 87 does not add a DiskANN `QuantCodec` implementation or
  TurboQuant search codec. Packet 005 is the Task 87 DiskANN Stop Condition,
  and packet 009 approves handing the blocked DiskANN codec work to Task 91.

The original universal "2x scoring-share per AM" target is not directly proven
by the available HNSW/IVF instrumentation and is not claimed here. The closeout
claim is narrower and matches reviewer-approved scope: shared CandidateBatch
plumbing plus recall-preserving off/on matrix evidence for SPIRE, IVF, and
HNSW on existing codecs, with DiskANN closed by accepted Stop Condition and
Task 91 handoff.
