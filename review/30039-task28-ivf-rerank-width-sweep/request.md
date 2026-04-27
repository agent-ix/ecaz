# Review Request: Task 28 IVF Rerank Width Sweep

## Summary

This packet records the first `rerank_width` sweep after commit `4d894bd`.
The sweep keeps the DBPedia-derived 10k x 1536 anchor, `nlists = 32`, and
`nprobe = 32`, and varies only the heap rerank frontier.

No DiskANN implementation or measurement is included.

## Fixture

- table: `task28_ivf_anchor10k1536_heap_corpus`
- exact top-10 source: `task28_ivf_anchor10k1536_heap_exact_top10` from packet 30038
- rows: 10,000
- dimensions: 1536
- index: `ec_ivf`, rebuilt one width at a time
- `nlists = 32`, `nprobe = 32`, `training_sample_rows = 2000`
- storage: `turboquant`
- rerank: `heap_f32`
- widths: 50, 200, 1000, 0 where 0 means full probed frontier
- cache state: normal local scratch state; not cold-cache controlled

## Results

All measured widths reached `1.0000` recall@10 on the first 20 ordered anchor
queries:

| rerank_width | returned | exact hits | recall@10 | 20-query materialization |
|---:|---:|---:|---:|---:|
| 50 | 200 | 200 | `1.0000` | `00:03.704` |
| 200 | 200 | 200 | `1.0000` | `00:03.860` |
| 1000 | 200 | 200 | `1.0000` | `00:04.754` |
| 0/full | 200 | 200 | `1.0000` | `00:14.107` |

Latency loop:

| rerank_width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|
| 50 | `180.608 ms` | `183.371 ms` | `183.742 ms` | `180.817 ms` |
| 200 | `189.890 ms` | `194.932 ms` | `195.676 ms` | `190.830 ms` |
| 1000 | `232.622 ms` | `235.727 ms` | `236.945 ms` | `233.192 ms` |
| 0/full | `702.929 ms` | `715.203 ms` | `715.447 ms` | `701.786 ms` |

Index build time was stable across the width rebuilds because width is scan-only:

| rerank_width | build time |
|---:|---:|
| 50 | `00:25.007` |
| 200 | `00:24.712` |
| 1000 | `00:24.618` |
| 0/full | `00:24.677` |

## Interpretation

On this local 10k x 1536 slice, a narrow `rerank_width = 50` recovers the same
`1.0000` recall@10 as full-frontier heap rerank while cutting p50 from about
`703 ms` to about `181 ms`. That is the first useful tuning lane: keep IVF
routing broad enough to include the right candidates, then exact-rerank a small
frontier.

This does not prove the width floor for larger corpora. It does show that
full-frontier heap rerank is unnecessary for this anchor and gives us a concrete
starting width range.

## Next Slice Recommendation

Run the next sweep over `nprobe x rerank_width` before increasing corpus size:

- `nprobe`: 4, 8, 16, 32
- `rerank_width`: 25, 50, 100, 200

The target is the lowest p95 point that keeps recall@10 close to the full-probe
heap-rerank oracle on the same 20-query anchor, then expand to more queries.

## Artifacts

- `artifacts/pg18-ivf-anchor10k1536-rerank-width-sweep.sql`
- `artifacts/pg18-ivf-anchor10k1536-rerank-width-sweep.log`
- `artifacts/manifest.md`

## Validation

Code validation before this packet:

- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `git diff --check`
