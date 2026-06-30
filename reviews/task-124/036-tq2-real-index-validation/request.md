# Task 124 Packet 036: TQ2 real-index validation

## Summary

This packet addresses reviewer feedback on packet 035: the TQ2 SIMD result from
packet 032 was a scorer microbenchmark only. I reran the existing real-index
TQ2 suite at 10k / 50k / 100k with current HEAD installed in PG18.

Result: current HEAD now emits in-engine TurboQuant scorer counter rows for the
real `rerank_format=turboquant2` IVF path, but TQ2 recall is unchanged from
packet 008 and still broken at 50k/100k. Therefore the packet-032 `-86%`
microbenchmark result must not be recorded as a usable TQ speedup.

## Validation

Passed:

- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `ecaz bench suite audit`: 12 steps
- `ecaz bench suite run`: 12 succeeded / 0 failed
- `ecaz bench suite status`: 12 completed / 0 failed / 0 stale
- `ecaz bench suite report`: generated

Config: `artifacts/task124-tq2-post-simd-suite.json`.
Artifact dir: `artifacts/tq2-post-simd-suite/`.

## Recall Versus Packet 008

The post-kernel recall is identical to the prior real-index TQ2 packet.

| Scale | nprobe | Packet 008 recall@10 | Packet 036 recall@10 | Packet 036 NDCG@10 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.9770 | 0.9770 | 0.9995 |
| 10k | 64 | 0.9770 | 0.9770 | 0.9995 |
| 50k | 32 | 0.8050 | 0.8050 | 0.9928 |
| 50k | 64 | 0.8050 | 0.8050 | 0.9928 |
| 100k | 32 | 0.7490 | 0.7490 | 0.9854 |
| 100k | 64 | 0.7550 | 0.7550 | 0.9879 |

## In-Engine Scorer Evidence

Packet 008 had only RabitQ block-kernel rows in the real TQ2 latency run. With
current HEAD, the same real-index TQ2 path now also reports
`quant=turboquant_qjl` rows:

| Scale | nprobe | TQ SIMD candidates | TQ SIMD elapsed | TQ scalar candidates | TQ scalar elapsed |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 9,600 | 2.606338 ms | 400 | 1.421123 ms |
| 10k | 64 | 9,600 | 2.703623 ms | 400 | 1.480047 ms |
| 50k | 32 | 9,600 | 2.683827 ms | 400 | 1.613960 ms |
| 50k | 64 | 9,600 | 2.756212 ms | 400 | 1.598001 ms |
| 100k | 32 | 9,600 | 2.734085 ms | 400 | 1.710623 ms |
| 100k | 64 | 9,600 | 2.825034 ms | 400 | 1.777504 ms |

This confirms the SIMD TQ2 scorer is exercised inside real IVF latency, but it
also shows a remaining scalar tail of 400 candidates per 100-query run.

## Latency Context

The real-index TQ2 latency improved modestly against packet 008, but the recall
gate still fails:

| Scale | nprobe | Packet 008 p50 | Packet 036 p50 | Packet 008 p95 | Packet 036 p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.93 ms | 0.70 ms | 1.05 ms | 0.82 ms |
| 10k | 64 | 1.39 ms | 1.22 ms | 1.49 ms | 1.33 ms |
| 50k | 32 | 2.57 ms | 2.43 ms | 2.84 ms | 2.68 ms |
| 50k | 64 | 4.88 ms | 4.83 ms | 5.06 ms | 4.96 ms |
| 100k | 32 | 5.22 ms | 5.03 ms | 5.79 ms | 5.54 ms |
| 100k | 64 | 9.53 ms | 9.26 ms | 9.75 ms | 10.5 ms |

## Decision

TQ2 SIMD is now workload-exercised, but it is not a usable TQ speedup because
real-index recall remains broken at 50k/100k. The only validated in-engine TQ
speedup from the current scorer pass remains the 4-bit TQ cascade improvement
recorded in packet 035.

Task 124 remains open for the remaining reviewer gap: reduced-dimension TQ still
needs a real format/reloption and real-index recall + latency + scorer-elapsed
validation before its microbenchmark figures can count as results.
