# Feedback: 648 Parallel Concurrent DSM 50k Measurement

## Verdict: Accept (measurement valid, packet incomplete)

**This packet has no `request.md`.** The artifacts exist but there is no review
envelope. Future measurement packets must include a `request.md` with fixture
description, result summary, interpretation, and validation commands — same
format as packets 622, 626, 629, etc.

## Result (from log)

50,000 rows × 64 dimensions, `ecvector`, default `turboquant`, `m=6`,
`ef_construction=40`, 4 workers requested/launched.

| Path | Wall time | Graph phase |
|---|---|---|
| Serial | 31,307 ms | 29,350 ms |
| Parallel serial-graph (ingest parallel, graph serial) | 28,717 ms | 27,703 ms |
| Parallel concurrent DSM graph | **11,607 ms** | **10,532 ms** |

**62.9% wall-clock reduction** vs serial. **64.1% graph phase reduction**
(29,350 ms → 10,532 ms). This is the headline result for task 19.

With 4 workers + 1 leader = 5 participants, 29.3s serial graph → 10.5s is
roughly 2.8× speedup on the graph phase — reasonable for 5 participants with
lock contention at m=6.

## heap_ingest_us Discrepancy

The concurrent DSM path reports `heap_ingest_us = 11,467 ms` vs 617 ms for the
parallel serial-graph path. This inflated value needs explanation in the
`request.md`. The heap ingest in the concurrent DSM path runs the same shm_mq
workers as before; the counter likely captures something different in the new
two-phase coordinator, or the measurement surface changed. Clarify before the
next packet.

## index_tuples

Both serial and concurrent DSM show `index_tuples = 49,982`. Same value across
all three paths — correct. Index integrity is not compromised by concurrent
graph assembly.

## Next Step

A recall validation packet is needed before the GUC default changes or this
path is promoted. The measurement proves speed; recall quality under concurrent
insertion order must be verified against the serial-build threshold.
