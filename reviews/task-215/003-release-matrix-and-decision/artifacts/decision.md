# Task 215 release A/B decision

## Decision

STOP. Do not promote BW64/H8 as the normal-release defaults. Restore and ship
BW4/H100. The candidate completed all six release arms and passed the physical
topology, owner engagement, storage, and release-profile gates, but it failed
the release contract's recall-stability and latency-Pareto requirements.

The candidate used the same session GUC `candidate_heap_limit=32` as the
control, but the existing runtime clamp made its effective heap L=64 under
BW64. The measured result is therefore recorded as BW64/H8/L64 effective,
with 128 production-derived head seeds, versus BW4/H100/L32 with 32 seeds.

## Paired physical results

| scale | arm | recall (95% CI) | mean ms | p50 / p95 / p99 / max ms | physical bytes |
|---|---|---:|---:|---:|---:|
| 10k | control BW4/H100/L32 | 0.9990 (0.9964–0.9997) | 18.80 | 19.00 / 22.50 / 22.90 / 23.10 | 242,753,536 |
| 10k | candidate BW64/H8/L64 | 0.9995 (0.9972–0.9999) | 22.60 | 21.90 / 28.90 / 33.40 / 35.90 | 242,745,344 |
| 50k | control BW4/H100/L32 | 0.9545 (0.9445–0.9628) | 20.80 | 20.70 / 24.00 / 24.40 / 24.50 | 1,242,750,976 |
| 50k | candidate BW64/H8/L64 | 0.9900 (0.9846–0.9935) | 29.00 | 28.90 / 33.90 / 35.50 / 36.20 | 1,242,742,784 |
| 100k | control BW4/H100/L32 | 0.9280 (0.9158–0.9385) | 21.40 | 20.50 / 26.00 / 26.90 / 27.10 | 2,496,651,264 |
| 100k | candidate BW64/H8/L64 | 0.9815 (0.9746–0.9865) | 31.60 | 31.40 / 39.20 / 42.70 / 44.00 | 2,496,659,456 |

Candidate mean latency was slower by 20.2%, 39.4%, and 47.7% at 10k, 50k,
and 100k respectively. Recall also changed materially at 50k and 100k; the
candidate is not recall-equivalent. The candidate's recall increase was
0.0005, 0.0355, and 0.0535 at 10k, 50k, and 100k respectively, but it came
with latency increases of 3.80, 8.20, and 10.20 ms. This is a recall/latency
tradeoff, not a Pareto improvement: the candidate did not dominate the
control, and the release contract requires recall equivalence unless the
trade is explicitly accepted. This decision explicitly rejects that higher-
recall trade under the recall-equivalence clause; a separate higher-recall
lane would be needed to decide whether that quality gain is worth the latency
cost. Storage differences were under 0.004% in these paired runs and do not
offset the latency/recall failure.

## Reconciliation with Task 206

The accepted Task 206 absolute latency rows (roughly 194–231 ms) are not
directly comparable to these 22.6–31.6 ms release rows. Task 206 measured
`top_k=200` with `candidate_heap_limit=200`, while Task 215 measured `top_k=10`
with effective `L=64` under BW64. Task 206 also used a different source/build
and diagnostic scan-notice configuration. Both used warm-cache,
single-concurrency sampling, and their per-scale query hashes match; neither
packet claims a concurrent-load difference. The top-k/L materialization work
surface is therefore the primary recorded confounder, but the artifacts do not
isolate it from the build difference. See
`artifacts/reconciliation-206.md` for the durable comparison and the rule for
which environment generalizes to release decisions.

## Entry-gate accounting

The standalone Task 208/210 entry-gate evidence was skipped in this packet.
The release matrix did collect its own topology, owner-engagement, storage,
and NFR rows, but those rows are not being represented as a substitute for
the required accepted Task 208/210 evidence. This omission does not change the
STOP verdict; it prevents any promotion claim beyond the measured matrix.

All six arms were normal PG18 release builds at source SHA
`ea51a9c8bdce1f412652ac743ae0d055af8daa76`, with three sharded owners, no
coordinator full-graph replica, no attribution feature, and no Task 216
candidate. The first stale-schema attempt is superseded diagnostic output;
the cited r2 run passed release preflight unanimously on all six arms.

## Rollback

The source defaults and default test are restored to BW4/H100 in the following
rollback checkpoint. Existing SQL/session rollback remains:

```sql
SET ec_distann.beam_width = 4;
SET ec_distann.hop_rounds = 100;
```

No index rebuild, persisted-format change, or Task 216 implementation is
authorized by this packet. Task 216 remains a separately scoped owner-side
attribution lane.
