# Task 28 IVF Current Gate Status After 990k

## Scope

This packet summarizes the current Task 28 merge-gate status after the A2 scale packet and the 990k IVF A9 packet.

No new measurements are introduced here; it consolidates the pushed packet evidence so the next reviewer pass has a short map.

## Status

| gate | current status | evidence |
|---|---|---|
| A1 cost model audit | done | packet 30076 |
| A2 streaming vacuum | done for code and 1M scale evidence | packets 30079, 30129 |
| A3 vacuum compaction/reuse | materially improved, not a full shrink claim | packets 30080, 30125 |
| A4 typed exact-score dispatch | done | packet 30102 |
| A5 `ProdQuantizer::cached` cache-key audit | done | packet 30102 |
| A6 planner matrix | done, with mixed-predicate corner called out | packet 30077 |
| A7 score-bound pruning | done for PQ-FastScan selected path | packets 30116, 30117 |
| A8 PQ-FastScan + RaBitQ wiring | done | packets 30081, 30082 |
| A9 100k+ IVF scale | IVF side now covered at 100k and 990k | packets 30126, 30130 |
| A10 quantizer recommendation | needs refresh with 990k included | packet 30127 plus 30130 |

## Updated Interpretation

The 990k IVF selected point is now measured:

- build: 33:53.835 for the IVF index after a 5:50.927 corpus copy
- size: 177 MB
- recall@10: 0.9860
- recall@100: 0.9509
- latency p50/p95/p99: 1029.2 / 1169.1 / 1224.4 ms
- latency HWM: 162636 KB

This is enough to say the current PQ-FastScan g8 IVF path has reasonable recall on the 990k anchor shape. It is not enough to call the current operating point latency-competitive.

## Next IVF Slice

The next slice should stay focused on IVF latency:

- Use the existing 990k isolated surface instead of rebuilding it.
- Sweep lower `nprobe` values and rerank widths on the same surface to find a latency/recall frontier below 1s/query.
- Add score-volume counters to quantify how many postings/candidates are scored and reranked per query.
- Treat benchmark harness exact-truth caching as a measurement-efficiency improvement, because 990k recall runs spend most wall time rematerializing the raw corpus.

Fresh broad HNSW comparison should not be the next step unless specifically requested; the immediate blocker exposed by 30130 is IVF scan latency at the selected point.

## Validation

- Synthesis only; source packets are cited above.

## Artifacts

- `artifacts/manifest.md`
