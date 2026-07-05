# Review Request: Task 28 IVF Full-Probe Scorer Alignment

## Summary

This packet explains the `nprobe = nlists` recall gap from packet 30036.

On the 10k x 1536 DBPedia anchor slice, full-probe IVF scores all 10,000
posting candidates but matches only `0.9200` recall@10 versus SQL exact
compressed scoring. This packet materializes exact SQL top-200 and IVF
full-probe top-200 for the same 20 queries and compares ranks and SQL scores.

No DiskANN work is included.

## Result

Fixture:

- table: `task28_ivf_anchor10k1536_corpus`
- rows: 10,000
- dimensions: 1536
- index: `task28_ivf_anchor10k1536_n32_idx`
- index size: `9,379,840` bytes
- storage: `turboquant`
- rerank: `off`
- IVF setting: `ec_ivf.nprobe = 32` for `nlists = 32`

Top-10 overlap:

| metric | value |
|---|---:|
| exact top-10 rows | 200 |
| IVF full-probe top-10 hits | 184 |
| recall@10 | 0.9200 |
| exact top-10 rows missing from IVF top-200 | 0 |
| worst IVF rank for exact top-10 rows | 14 |

The full-probe gap is therefore not candidate reachability. Every exact top-10
row appears in the IVF top-200; the 16 misses are rank-boundary ordering
differences where exact top-10 rows are demoted to IVF ranks 11-14.

Rank-boundary score drift:

| metric | value |
|---|---:|
| demoted exact top-10 rows | 16 |
| best demoted IVF rank | 11 |
| worst demoted IVF rank | 14 |
| average SQL score gap vs an extra IVF top-10 row | `0.0029758047312498093` |
| max SQL score gap | `0.007695481` |

Timing in this local warm-cache packet:

| step | time |
|---|---:|
| exact SQL top-200 materialization, 20 queries | `01:28.093` |
| IVF full-probe top-200 materialization, 20 queries | `00:05.163` |

## Interpretation

Full-probe IVF is doing the useful structural work: it reaches all candidates
and retrieves the exact top-10 rows into the near frontier. The remaining
`8pp` recall@10 gap is scorer/order drift at the final rank boundary.

That makes the next improvement path narrower:

- keep IVF routing/probe behavior as viable;
- do not tune centroids first to fix the full-probe gap;
- add or wire a stronger final rerank path for IVF, or explicitly document
  `rerank = off` as approximate even at full probe.

## Artifacts

- `artifacts/pg18-ivf-fullprobe-scorer-alignment.sql`
- `artifacts/pg18-ivf-fullprobe-scorer-alignment.log`
- `artifacts/manifest.md`

## Validation

Packet-only change; no code changed.

- `git diff --check`

## Next Slice Recommendation

Implement the smallest IVF rerank-capable path that can re-order the candidate
frontier after compressed posting-list scoring. A good first target is a
source-column or heap-f32 rerank mode for the top `rerank_width` candidates,
then rerun this same full-probe packet expecting the top-10 gap to close.
