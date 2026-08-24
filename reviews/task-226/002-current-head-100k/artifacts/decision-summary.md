# Task 226 current-head BW8 100k decision summary

Execution extension: `a1f1584966011ca7c16175fe91f8efc302c8cf25`,
release profile on all three PG18 owners. SuiteConfig SHA-256:
`faec8932e937ea12c85e201fa6a3601dc561572cbc687887b72ec0194a5f11f3`.
Both successful fixtures used `ec_real_100k`, 200 held-out queries, top-k 10,
L32, H100, RaBitQ neighbor scoring, lazy-10 materialization, one build shard,
and a fixed 4,096-entry persisted sharded head. Only traversal beam width
changed between the BW4 control and BW8 candidate.

## Production decision run

The A/A prediction artifacts are byte-identical (SHA-256
`84f3ee959c59b8541cb7347cb5b9525624d4bab9b77b440c6d3dabb24a6308db`),
and both A/A arms measured recall 0.9285. All four variants used generation
identity
`0200eca4b2f4257e985cc998a5c91d6123c7bdccb067a719e6977c78e9ada0c32120`.

| Variant | Recall@10 | Mean ms | p50 ms | p95 ms | p99 ms | Max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| BW4 control | 0.9285 | 16.40 | 16.20 | 19.00 | 19.70 | 20.00 |
| BW8 candidate | 0.9450 | 16.20 | 15.60 | 19.80 | 20.70 | 20.80 |

Paired recall: candidate wins 20, control wins 2, ties 178; candidate minus
control mean is +0.016500 with paired bootstrap 95% CI
`[+0.008000, +0.026500]`. Mean latency improves by 1.22% (0.20 ms), while
p95 regresses by 4.21%. This satisfies preregistered ADVANCE branch (b): the
paired recall lower bound is nonnegative and both mean and p95 latency changes
stay within the 5% envelope.

Storage is arm-invariant: 2,498,281,472 physical generation bytes,
831,782,912 graph-side bytes, 1,666,498,560 owner row-tier bytes, and zero
coordinator-resident unsharded bytes. Published owner rows are
33,195 + 33,432 + 33,373 = 100,000, with zero non-owned rows and zero
orphans.

## Full-metrics attribution run

This separate fresh fixture is diagnostic and is not used for the production
latency gate. It reproduced the recall direction: BW4 0.9275 versus BW8
0.9460, paired delta +0.018500 with 95% CI `[+0.009500, +0.029000]`
(21 candidate wins, 1 control win, 178 ties).

| Variant | Scan total mean ms | Traversal mean ms | Transport wait mean ms | Remote expand mean ms | Remote materialize mean ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| BW4 control | 14.917298 | 6.771611 | 3.259058 | 5.331008 | 5.716154 |
| BW8 candidate | 15.549734 | 6.878988 | 2.795992 | 5.067135 | 6.009163 |

BW8 therefore reduces traversal transport wait and remote-expand time, while
slightly increasing total scan and remote-materialization time. Instrumented
end-to-end latency was BW4 mean/p95 16.60/19.50 ms and BW8 17.40/20.90 ms;
those rows are diagnostic because stage counters perturb the production path.
The executor consumed 6.66 remote rows per BW4 scan and 6.58 per BW8 scan, so
the recall win is not explained by shipping a larger final payload.

## Disposition

`ADVANCE` the unchanged BW8 candidate to the preregistered fresh 10k/50k
confirmation matrix. This is not a default-policy change: Task 226 remains
review-open until the full matrix and outside review are complete.
