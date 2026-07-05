# Task 79 Packet 040: RaBitQ k=3 Block16 Candidate Surface

This packet tests the direct candidate-reduction hypothesis that block16 plus k=3 summaries can keep the multi-representative recall gain while cutting scored row candidates below the Task 79 gates.

The answer is mixed and does not close Task 79. It is a real candidate-surface win: `global1216` scores `3,877,368` candidates over 200 queries, meeting the strong `<=4.0M` target, and reaches recall@10 `0.9940`. However, p50 is `52.643 ms`, far above the `<=45 ms` target. Higher caps improve recall but make latency worse.

| row | global blocks | candidates | p50 | p95 | recall@10 | gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| k3 block16 | 1024 | 3,265,373 | 50.658 ms | 58.404 ms | 0.9920 | fail_recall_p50 |
| k3 block16 | 1216 | 3,877,368 | 52.643 ms | 62.906 ms | 0.9940 | fail_p50 |
| k3 block16 | 1280 | 4,081,213 | 54.166 ms | 62.936 ms | 0.9950 | fail_p50 |
| k3 block16 | 1536 | 4,897,128 | 55.530 ms | 61.944 ms | 0.9960 | fail_p50 |
| k3 block16 | 1664 | 5,304,964 | 56.407 ms | 67.552 ms | 0.9965 | fail_candidate_p50 |

Compared with packet 035 block32 k=3, block16 proves the candidate gate can be satisfied with multi-representative recall. The blocker shifts to summary scoring and object-read cost: every row uses fewer candidate rows, but block16 doubles the summary population scored before global selection. Object bytes are flat at `14,967,100,324` across caps because the same `19,200` routed leaf pids are read; candidate rows change only after global block selection.

Evidence:

- `artifacts/manifest.md`
- `artifacts/compact-results.tsv`
- `artifacts/suite-run.log`
- `artifacts/results.jsonl`
- `artifacts/suite-report.log`
- `artifacts/k3-block16-cluster-mean.patch`

The temporary k=3 builder patch was reverted after measurement. Local PG18 was restored to the clean fast-path backend SHA `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`.

Next local direction: do not increase cap or keep shrinking blocks. The useful path is to avoid scoring all block16 summaries, probably via a cheap first-stage block shortlist or a leaf-local top-M block budget that preserves the `global1216` candidate surface while reducing the `19,200` routed-leaf object/read-summary work.
