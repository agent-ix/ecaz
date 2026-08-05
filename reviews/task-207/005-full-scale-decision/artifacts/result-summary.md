# Task 207 result summary

> Packet `../006-re-review-corrections/` supersedes the owner-oracle table and
> effective-seed wording below. The release construction decision remains,
> but owner rows do not establish head membership or overlap@k.

Release decision-arm results. Latency is mean/p50/p95/p99/max ms; storage
amplification is the cluster graph-side normalized ratio.

| scale | construction | recall | latency | graph-side amplification |
|---|---|---:|---|---:|
| 10k | stitched_bfs | 0.9658 | 191.10; 191.6/205.7/219.8/231.9 | see source summary |
| 10k | partition_union | 0.9568 | 182.80; 183.9/200.6/205.5/205.6 | see source summary |
| 50k | stitched_bfs | 0.9051 | 200.80; 198.5/221.9/235.8/245.7 | see source summary |
| 50k | partition_union | 0.8997 | 189.90; 188.9/203.8/217.1/222.2 | see source summary |
| 100k | stitched_bfs | 0.9090 | 197.90; 198.4/212.3/217.9/221.2 | 1.351987 |
| 100k | partition_union | 0.9128 | 204.00; 201.9/225.7/264.1/282.4 | 1.352000 |

At 100k, physical generation bytes are 2,497,159,168 (stitched) and
2,497,167,360 (union), with the same 854,810,624 single-index reference.
Build time is 873,229 ms versus 892,955 ms at 100k and 406,963 ms versus
408,927 ms at 50k (stitched versus union). The candidate therefore has no
stable Pareto advantage.

The owner diagnostic uses the identical fixed four-shard graph and reports
membership recall, not only end-to-end recall. Its final per-scale values are
recorded below; this lane is not used to promote a default or compare
instrumented latency with the clean release matrix.

| scale | construction | membership recall | owner-control latency |
|---|---|---:|---|
| 50k | stitched_bfs | 0.7080 | 1215.80 ms mean; p50 1163.80, p95 1387.80, p99 1416.50, max 1423.60 |
| 50k | partition_union | 0.7080 | 1216.60 ms mean; p50 1171.60, p95 1382.70, p99 1395.90, max 1404.10 |
| 100k | stitched_bfs | 0.7893 | 2443.60 ms mean; p50 2489.70, p95 2558.40, p99 2588.60, max 2596.50 |
| 100k | partition_union | 0.7893 | recall artifact captured; latency/storage follow-ons not used |

All completed owner arms passed remote-owner activation. The final union
100k recall artifact was captured before its wrapper stopped producing child
artifacts; the stopped generated cluster was removed. The clean release A/B
matrix above remains complete and is the only production decision evidence.
