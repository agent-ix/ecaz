# Task 79 Packet 041: RaBitQ Block16 Two-Stage Summary Shortlist

This packet tests the local-only follow-up from packet 040 feedback: avoid scoring all block16 k=3 summary representatives by doing a cheap first-pass summary score and then fully rescoring only a shortlist before applying the same global block cap.

The result is negative. The two-stage representative shortlist does not reduce candidates, routed leaves, object bytes, or p50. It keeps recall stable, but the best p50 row is effectively the same as the full-summary baseline and still misses the Task 79 latency gate.

| row | candidates | p50 | p95 | recall@10 | production total p50 | gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| full | 3,877,368 | 52.135 ms | 63.231 ms | 0.9940 | 48 ms | fail_p50 |
| fp1 | 3,877,368 | 53.790 ms | 61.469 ms | 0.9940 | 49 ms | fail_p50 |
| fp1_rescore1536 | 3,877,368 | 52.226 ms | 58.537 ms | 0.9940 | 48 ms | fail_p50 |
| fp1_rescore2048 | 3,877,368 | 52.303 ms | 61.709 ms | 0.9940 | 48 ms | fail_p50 |
| fp1_rescore3072 | 3,877,368 | 52.801 ms | 60.361 ms | 0.9940 | 48 ms | fail_p50 |
| fp2_rescore2048 | 3,877,368 | 52.054 ms | 57.878 ms | 0.9940 | 48 ms | fail_p50 |

All rows keep the same `19,200` routed leaf pids and `14,967,100,324` object bytes. This closes the cheap two-stage representative shortlist axis: reducing summary representative scoring alone is not where the remaining block16 latency is going.

The temporary patch was intentionally not promoted. It is preserved as `artifacts/two-stage-summary-shortlist.patch`; local PG18 was restored afterward to the clean backend SHA `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`.

Evidence:

- `artifacts/manifest.md`
- `artifacts/compact-results.tsv`
- `artifacts/results.jsonl`
- `artifacts/suite-run.log`
- `artifacts/suite-report.log`
- `artifacts/two-stage-summary-shortlist.patch`
- `artifacts/cargo-test-prefix-representative.log`
- `artifacts/cargo-test-full-rescore.log`

Interpretation: packet 040 proved the candidate gate can be met with block16/k=3, but packet 041 shows the local cheap-shortlist idea does not recover latency. The next credible local work is either a real k=3 RaBitQ scoring-kernel optimization or a larger architecture change that reduces routed leaf/object read surface before summaries are scored.
