---
task: 192
packet: 007-full-scale-decision
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 192 full-scale decision: PROMOTE

The bounded retained-generation row-schema cache passes the complete
10k/50k/100k decision matrix and the separate epoch-safety gate. Task 192
closes **PROMOTE** to a productionization follow-up; this packet does not turn
the benchmark-only switch on in production.

## Full-scale A/B

Each scale used one byte-identical three-owner physical generation. Both arms
used the production trained head, RaBitQ neighbor codes, lazy10 materialization,
BW=4/H=100, 200 recall queries, and 10 warmups + 50 measured latency queries.
The sole difference was cached versus live owner row-schema resolution.

| Scale | recall uncached/cached | warm mean uncached | warm mean cached | delta | p95 uncached/cached | physical bytes uncached/cached |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 0.9990 / 0.9990 | 24.70 ms | 19.30 ms | -5.40 ms (-21.9%) | 28.60 / 22.50 ms | 242,745,344 / 242,745,344 |
| 50k | 0.9685 / 0.9685 | 23.50 ms | 19.80 ms | -3.70 ms (-15.7%) | 26.70 / 22.80 ms | 1,242,734,592 / 1,242,734,592 |
| 100k | 0.9625 / 0.9625 | 23.70 ms | 19.70 ms | -4.00 ms (-16.9%) | 26.60 / 22.80 ms | 2,496,659,456 / 2,496,659,456 |

The stage signature is causal and consistent:

| Scale | owner open/validate uncached | cached | remote materialize uncached | cached | owner payload SQL uncached | cached |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 7.818 ms | 0.026 ms | 11.132 ms | 6.994 ms | 9.069 ms | 8.830 ms |
| 50k | 6.708 ms | 0.023 ms | 10.492 ms | 6.976 ms | 8.856 ms | 8.819 ms |
| 100k | 6.889 ms | 0.024 ms | 10.448 ms | 6.945 ms | 8.757 ms | 8.737 ms |

Recall, seed IDs, request/result work, and storage are identical. Payload SQL
is flat while the targeted open/validate component disappears and end-to-end
mean and tails improve at every scale.

## Safety and provenance

Packet 006's real PG18 multi-epoch drill passed: a successor publication
replaces the same-index cache entry; the retained predecessor remains readable;
reclaim evicts it and the stale fingerprint fails as `EC_GENERATION_MISSING`.
The cache remains backend-local, generation-fingerprinted, relcache-invalidated,
and bounded to four indexes with one fingerprint per index.

The suite completed 3/3 steps with zero failed, missing, or stale artifacts.
All three owners unanimously reported release extension
`0ef3a3b9e96e9357953d983ad4aa6338672745e0`; installed and target binaries
were byte-identical 24,212,096-byte files. The runner SHA is the later
config-only commit `8ae76282689daf7087524718229a870e9d82d0d7`.

## Disposition

PROMOTE `MAT-37`/`MAT-38` as one separately reviewed productionization task.
The production slice must preserve the packet-006 fencing contract and remove
the benchmark GUC rather than silently changing normal behavior in this task.
Task 193 may now use the unchanged production arm as its separately attributed
baseline.
