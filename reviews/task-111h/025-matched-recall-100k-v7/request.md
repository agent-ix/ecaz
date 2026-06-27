# Task 111h / Packet 025 Review Request: 100k v7 Matched-Recall Analysis

## Summary

This packet requests review for a derived matched-recall analysis of packet 024's
post-v7 100k rerank suite.

No new benchmarks were run. The analysis joins packet 024 recall, latency, and
storage JSONL rows, then selects the lowest-p50 row per format for recall targets
`0.90`, `0.93`, `0.95`, `0.97`, and `0.99`.

Main artifact:

- `artifacts/matched-recall-100k-v7.md`

Provenance:

- `artifacts/manifest.md`

## Result Summary

On the 100k post-v7 warm-cache local data:

- At target recall `0.90`, RaBitQ8 and TurboQuant have a small p50 edge over
  source f32, but with much larger ec_ivf indexes and lower selected-row recall.
- At target recall `0.93`, source f32 is already faster than f16, RaBitQ4, and
  RaBitQ8; TurboQuant is close but still slower and roughly 5x larger in index
  bytes.
- At target recall `0.95`, source f32 is faster than every compact quantized
  format that reaches the target; RaBitQ4 does not reach it.
- At target recall `0.97` and `0.99`, only source f32 and f16 reach the target.
  Source f32 is faster and has a roughly 13x smaller ec_ivf index than f16.

## Interpretation

This post-v7 100k matched-recall slice supports source f32 as the warm-cache
local reference path. It does not support promoting the current compact
index-side formats for this workload.

It also should not be read as an abandon decision for every compact format:
Task 111h still requires cold/remote storage evidence, table-owned storage or a
replacement decision, legacy `0x2A` attribution, and the full cross-scale final
decision table.

## Review Ask

Please review whether:

- the selection rule is the right one for the Task 111h matched-recall gap,
- the derived table is supported by packet 024's committed JSONL artifacts,
- the interpretation is appropriately limited to 100k post-v7 warm-cache local
  data,
- the next evidence slice should be cold/remote for source f32 vs the surviving
  compact candidates, or the table-owned/legacy-0x2A attribution work first.
