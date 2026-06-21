---
task: 118
packet: reviews/task-118/018-final-table-extractor
checkpoint_sha: 95dda99de3ba2d4d240537106a5ee8ba7041e8d4
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Final Table Extractor

## Scope

This checkpoint adds a packet-local `jq` extractor for the final Task 118
decision table.

The final packet 006 closeout has to join several result kinds by scale,
format, and build path:

- recall rows at `ef_search=200`;
- HNSW frontier containment and rerank-boundary counters;
- HNSW score-correlation summaries;
- total storage rows.

The extractor reads one or more `ecaz bench suite` `results-*.jsonl` files and
emits a TSV table with the packet 006 decision columns plus blank
`Dominant loss stage` and `Next action` fields for the final interpretation.

## Validation

Validated against the committed 10k source/compressed recall+storage artifacts
from packet 006 and the current-head AMD frontier/score artifacts from packet
016.

Command:

```bash
jq -sr -f reviews/task-118/018-final-table-extractor/artifacts/task118-final-table.jq \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-compressed-rerun.jsonl \
  reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-frontier-current-head-amd.jsonl \
  reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-score-current-head-amd.jsonl
```

Artifact: `artifacts/final-table-extractor-10k-amd-validation.txt`

The validation output contains six 10k rows: TurboQuant, PqFastScan, and RaBitQ
for source-build and compressed-build lanes.

## Remaining Task 118 Closeout Work

After the Intel 10k/50k/100k results land, run this extractor over
`results-10k-intel.jsonl`, `results-50k-intel.jsonl`, and
`results-100k-intel.jsonl`, then fill the two interpretation columns in packet
006.
