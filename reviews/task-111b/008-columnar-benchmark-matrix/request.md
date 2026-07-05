# Task 111b Columnar Benchmark Matrix

## Scope

This packet requests review of the final Task 111b columnar benchmark matrix
and its supporting raw-page capacity fix.

Code fix already committed:

- `9cdff9976 Task 111b: honor columnar raw page guard`

The final measurement head is:

- `376da5eba72d1e1abe44e86399bd9c32fe8badbf`

## What Changed During Measurement

The first complete suite attempt exposed a writer bug:

```text
ERROR: ec_ivf columnar page payload 8166 exceeds raw capacity 8160
```

The fix changes `columnar_page_chunk_lengths` to use
`columnar_frozen_list_raw_page_capacity(page_size)` rather than the unguarded
usable page bytes. Focused tests passed:

- `columnar_frozen_list_raw_page_chunks_obey_guard_capacity`
- `columnar_frozen_list_raw_pages_keep_all_column_items_whole`
- `columnar_frozen_list_raw_pages_match_header_block_range`

## Benchmark Coverage

Final suite status:

```text
completed=50 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The suite measured 10 isolated columnar cells:

- 50k and 100k real-corpus fixtures.
- TurboQuant plus RaBitQ quant bits 1, 2, 4, and 8.
- Per cell: load, recall, warm latency, storage, and nprobe-32 EXPLAIN.
- `nlists=64`, `nprobe=32`, `training_sample_rows=10000`.
- `dense_posting_blocks=0`, `columnar_frozen_lists=1`.
- Scan GUCs: dense coalescing on, typed views off.

## Key Results

At nprobe 32:

| scale | quant | recall@10 | latency mean | index size |
| --- | --- | ---: | ---: | ---: |
| 50k | TQ | 0.9420 | 14.4 ms | 43.4 MiB |
| 50k | rb1 | 0.7750 | 6.71 ms | 14.2 MiB |
| 50k | rb2 | 0.8840 | 65.4 ms | 23.9 MiB |
| 50k | rb4 | 0.9410 | 18.8 ms | 43.4 MiB |
| 50k | rb8 | 0.9460 | 15.7 ms | 82.5 MiB |
| 100k | TQ | 0.9370 | 35.9 ms | 83.4 MiB |
| 100k | rb1 | 0.7630 | 13.2 ms | 24.8 MiB |
| 100k | rb2 | 0.8670 | 131.9 ms | 44.4 MiB |
| 100k | rb4 | 0.9290 | 39.2 ms | 83.4 MiB |
| 100k | rb8 | 0.9390 | 45.6 ms | 161.5 MiB |

EXPLAIN shows the scan path stays fully columnar and wide-batch:

- Row/dense postings visited: 0 in all EXPLAIN cells.
- Columnar frozen lists visited: 32 in all EXPLAIN cells.
- Dense coalesced flushes: 109 at 50k, 178 at 100k.

## Interpretation

Columnar preserves recall and avoids the small-batch TQ problem. It also
validates the reviewer-requested shape of keeping list metadata once, then
feeding payload into the coalesced scorer.

It is not yet the durable storage winner. Compared with Task 111a, columnar is
smaller than row in all measured cells, but original dense remains smaller in
every measured cell. Latency is also mixed: TQ is close to row but slower than
111a dense-a/dense-b, rb1 is competitive, rb2 remains an outlier despite wide
batches, and rb4/rb8 are close to row but behind the best original-dense rows.

The packet conclusion is that 111b is a valid proof and measurement baseline,
but 111c/111d should focus on cheaper page layout / continuation geometry
rather than promoting this columnar format as-is.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary.md`
- `artifacts/task111b-columnar-suite.json`
- `artifacts/suite/suite-manifest.json`
- `artifacts/suite/results.jsonl`
- `artifacts/suite/results-report.jsonl`
- `artifacts/suite-run-r4.log`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/cargo-test-columnar-raw-capacity.log`
- `artifacts/suite/explain-*.log`
- `artifacts/suite/explain-*.sql`

The generated truth-cache JSON files are intentionally excluded from commit.

