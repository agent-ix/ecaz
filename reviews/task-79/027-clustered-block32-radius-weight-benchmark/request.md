# Review Request: Task 79 Clustered Block32 Radius-Weight Benchmark

## Scope

This is a local-only Task 79 measurement packet. No AWS was used. The suite tests whether clustered RaBitQ leaf blocks at 32 rows, plus the existing summary-radius scoring term, can directly reduce the SPIRE candidate surface while preserving recall.

Primary target remains RaBitQ. TurboQuant was not run in this packet.

## Evidence

- Suite config: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/suite-rabitq-clustered-block32-radius-weight.json`
- Manifest: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/manifest.md`
- Compact table: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/compact-results.tsv`
- Suite run log: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-run.log`
- Suite status: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-status.log`
- Suite report: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-report.log`

Suite status was clean:

```text
completed=13 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

The first recall-passing local RaBitQ point was:

```text
block32-global896-rw025: candidate_sum=5674919, p50=47.620 ms, p95=54.526 ms, recall@10=0.9900
```

For context:

- Unbounded block32 baseline in this packet: 15,506,227 candidates, p50 63.228 ms, recall@10 0.9975.
- Best lower-cap non-passing point: `global832/radius0.25`, 5,269,892 candidates, p50 48.496 ms, recall@10 0.9885.
- Higher-margin passing point: `global1024/radius0.25`, 6,483,892 candidates, p50 50.705 ms, recall@10 0.9915.
- Prior packet 025 recall-passing clustered block64 point: 9,525,502 candidates, p50 56.486 ms, recall@10 0.9930.

## Interpretation

Clustered block32 plus radius0.25 is a real candidate-surface improvement: it gets a local RaBitQ recall-passing point below 6M candidates and improves p50 versus the prior recall-passing block64 setting.

It is not enough to consider Task 79 solved. Recall still requires millions of candidates for 200 queries, and the recall gain from cap896 to cap1024 is small relative to the extra candidates. Combined with packet 026, this points to the same structural fix: add richer per-block representative information and score against it, rather than continuing cap-only or radius-weight-only tuning.

## Review Ask

Please review whether the packet evidence supports using `block32/global896/radius0.25` as the current local RaBitQ baseline, and whether the next code checkpoint should move to multi-representative per-block scoring.
