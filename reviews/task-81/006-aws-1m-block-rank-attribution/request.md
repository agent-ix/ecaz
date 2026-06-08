# Task 81 Packet 006: AWS 1M Block-Rank Attribution Attempt

## Summary

This packet records a failed/abandoned AWS 1M rank-attribution attempt after
Task 81 packets 003 and 005 both stayed at recall@10 `0.9832` on the retained
q500 1M surface.

The only useful diagnostic result is the q1 local-sequence offset probe:
`truth_id + 1` is the correct mapping for this retained SPIRE surface, and q1
has all 10 exact neighbors selected under `global1152`.

The broader q50 attribution path was abandoned:

- slicing the q500 truth cache to q50 failed descriptor validation because the
  query hash no longer matched the limited q50 query set;
- running without a truth cache streamed/scored the full 1M corpus for exact
  q50 truth and was cancelled;
- the raw SQL fallback also ran too long for a diagnostic query and was
  cancelled.

## Decision

Do not spend more Task 81 time on the retained q500 1M rank-attribution path
until the suite runner has a first-class way to reuse a validated q500 truth
cache for a limited prefix of queries or emit per-query rank diagnostics without
recomputing exact truth.

The next packet should use the corrected acceptance comparison: beat the Task
79 accepted local/AWS 100k/q200 rows at the same recall/candidate point, rather
than treating the old full-leaf `15.5M` row as the success baseline.

## Evidence

- Manifest: `artifacts/manifest.md`
- Q1 offset probe: `artifacts/ssm-rank-offset-probe-q1-rerun-final4.json`
- q50 cache validation failure:
  `artifacts/ssm-cloud-bench-rank-attribution-q50-rerun2-fail.json`
- uncached q50 progress probe:
  `artifacts/ssm-progress-during-q50-nocache.json`
- cancellation/pause log:
  `artifacts/cloud-pause-after-cancelled-attribution.log`

## Reviewer Focus

1. Confirm this packet should be treated as negative diagnostic provenance, not
   a gate result.
2. Confirm it is reasonable to pivot the next Task 81 AWS packet to the Task
   79 q200 comparison baseline clarified by the user.
