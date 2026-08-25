# Task 222 isolated 100k decision

Decision: **ADVANCE** to the full 10k/50k/100k matrix.

| Metric | All-columns control | Projected candidate | Result |
| --- | ---: | ---: | ---: |
| Recall@10 / distinct recall | 0.9265 | 0.9265 | unchanged |
| Warm mean | 17.1 ms | 10.7 ms | -6.4 ms (-37.43%) |
| p50 | 16.9 ms | 10.5 ms | -6.4 ms |
| p95 | 19.8 ms | 12.8 ms | -7.0 ms |
| p99 | 20.4 ms | 12.9 ms | -7.5 ms |
| Maximum | 20.7 ms | 13.0 ms | -7.7 ms |
| Payload columns requested / scan | 33.30 | 6.66 | -80.0% |
| Payload bytes returned / scan | 167,404.76 | 66.60 | -99.9602% |
| Remote rows returned / scan | 6.66 | 6.66 | unchanged |
| Owner payload SQL work / scan | 7.862213 ms | 0.453321 ms | -94.23% |
| Owner endpoint work / scan | 8.144455 ms | 0.729461 ms | -91.04% |
| Physical generation bytes | 3,189,694,464 | 3,189,694,464 | unchanged |
| Owner graph-side bytes | 831,782,912 | 831,782,912 | unchanged |

The prediction files have the same SHA-256:
`156fc23a84231be13a193b9b7406181f5bef386941e6ad3535cdb5ef537e525b`.
Both arms share generation identity
`0200a5aee8e9e97631befa08cbbcf2ffa84e8512ea4e62ee9f08464787c75b3aef04`,
head-membership SHA-256
`fd5a185f1789cc153357fca331638730ef3c83f2beccfc6bcca69e19b6422d79`,
and query SHA-256
`a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.

The same 333 remote rows are returned over 50 timed scans in both arms. The
control requests five columns per row; the candidate requests only `id`. This
proves the ordering-only exemption excluded the 1,536-dimensional embedding
rather than accidentally measuring an `{id, embedding}` mask.

Materialization correctness passes for: fewer than one window, exactly one
window, more than one window, first-window rejection, multi-window rejection,
null payload, toasted projection/qual, mixed local/remote consumption, and
post-first-batch remote failure. Every comparable eager/candidate digest is
identical and every scenario reports zero duplicate requests.
