# Review Request: RaBitQ Bit-Width Dense Layout Sweep

## Scope

This packet adds the missing Task 111a benchmark evidence for RaBitQ bit widths
2, 4, and 8. It complements packet `004-all-dense-options-benchmark`, which
covered TurboQuant and rb1.

The suite measured both 50k and 100k fixtures across all six requested surfaces:
row postings, original dense, original dense with coalescing, original dense
with typed views, page-spanning packed dense, and page-spanning packed dense
with typed views.

## Result

The suite completed cleanly:

- `completed=180 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Artifact summary: `artifacts/summary.md`
- Suite config: `artifacts/task111a-rabitq-bitwidth-suite.json`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Structured results: `artifacts/suite/results.jsonl`
- Report results: `artifacts/suite/results-report.jsonl`
- Packet manifest: `artifacts/manifest.md`

At nprobe=32, recall is unchanged across storage surfaces for each bit
width/scale. The main latency/storage result is that the simpler dense layouts
are the durable winners for rb2/rb4/rb8:

- rb2 100k: dense-old 124.5 ms, dense-typed 130.6 ms, row 140.7 ms,
  dense-b-typed 135.4 ms, dense-b 141.5 ms.
- rb4 100k: dense-typed 32.4 ms, dense-a 35.0 ms, dense-b-typed 35.6 ms,
  row 38.6 ms, dense-b 40.7 ms.
- rb8 100k: dense-old 32.3 ms, dense-a/dense-typed 32.6 ms, dense-b-typed
  35.1 ms, row 43.7 ms.

Storage follows the same pattern. At 100k, original dense is smaller than row
for every tested RaBitQ bit width. The current page-spanning format is not the
best storage shape: rb2 page-spanning is roughly row-sized, rb4 page-spanning is
larger than row, and rb8 page-spanning is smaller than row but larger than
original dense.

## Notes for Reviewer

The selected EXPLAIN logs show the page-spanning packed path is exercised:

- 100k rb2 dense-b-typed: 1,314 logical groups assembled from 2,628 segments.
- 100k rb4 dense-b-typed: 1,323 logical groups assembled from 5,264 segments.
- 100k rb8 dense-b-typed: 1,327 logical groups assembled from 9,222 segments.

This supports the reviewer feedback that the durable page-spanning format should
store metadata once per logical group and make continuation segments payload-only
or mostly payload-only. The current implementation proves the path and counters,
but the measured page/copy/storage cost argues against keeping this physical
shape as the final format.

Please review whether this packet is enough to close the benchmark coverage gap
for "all options tested" across TQ plus RaBitQ `{1,2,4,8}` and whether the
storage-format conclusion in `summary.md` matches the evidence.
