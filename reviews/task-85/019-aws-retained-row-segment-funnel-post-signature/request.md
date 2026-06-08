# Review Request: AWS Retained Row Segment Funnel After Signature Update

Task: `plan/tasks/85-spire-product-scale-pareto-program.md`
Head SHA: `e07b4be5ee28ae74d85a7b4a601340307f0bb413`

## Summary

This packet reruns the retained AWS `1m`/q500 SPIRE block16/global1152 funnel
after packet 018 updated the retained DB function signature. It is the first
trustworthy AWS q500 row-segment evidence after the append-only snapshot fix.

## Key Results

Warm repeat, retained block16/global1152, nprobe 96, rerank width 25:

- recall@10: `0.9876`
- candidate_sum: `9,213,846`
- heap_rerank_sum: `12,500`
- p50/p95/p99/max latency: `227.388 ms` / `284.166 ms` /
  `297.164 ms` / `301.404 ms`
- selected row segment reads: `1,180,606`
- selected row segment bytes: `9,622,405,352`
- legacy row-object span: `304,802,815,448` bytes
- object-read time sum: `94,059,241,491 ns`
- summary-score time sum: `22,719,628,957 ns`
- candidate-score time sum: `27,865,902,960 ns`

The retained surface still matches the Task 79/81 candidate and rerank budget,
but the new counters show the actual selected row-segment payload is about
`9.62 GB` across q500, while the old row-object byte counter describes a much
larger enclosing object span. The current object-read bottleneck is therefore
read-call/segment fragmentation and layout locality, not simply the full
row-object byte span.

## Evidence

- `artifacts/suite-audit.log`: suite audit passed.
- `artifacts/aws-1m-retained-row-segment-funnel-post-signature-q500/results.jsonl`:
  top-line retained q500 recall, candidate, rerank, and latency rows.
- `artifacts/aws-1m-retained-row-segment-funnel-post-signature-q500/funnel-retained-global1152-q500-repeat-post-signature.jsonl`:
  per-query row-segment counters for the warm repeat run.
- `artifacts/cloud-status-final-after-retained-row-segment-post-signature.log`:
  AWS `1m` profile paused after the run.

## Next Workstream

Move object-read/physical-layout from instrumentation to implementation:
reduce selected row-segment read-call fragmentation or coalesce selected-block
payload reads while preserving the retained candidate set, recall, and
`heap_rerank_sum`.
