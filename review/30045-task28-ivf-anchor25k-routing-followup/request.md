# Review Request: Task 28 IVF 25k Routing Followup

## Summary

This packet reuses the 25k / 100-query exact truth table from packet 30044 and
checks three additional routing points. The goal is to see whether `nlists=64`
improves the 25k recall/latency frontier.

No DiskANN implementation or measurement is included.

## Fixture

- corpus table: `task28_ivf_anchor25k_corpus`
- corpus rows: 25,000
- dimensions: 1536
- query table: `task28_ivf_anchor10k1536_queries100`
- exact truth: `task28_ivf_anchor25k_exact100_top10`
- exact rows: 1,000
- exact queries: 100
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Results

Candidate points:

| nlists | nprobe | width | build | materialize | recall@10 | hits | index size |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 32 | 28 | 25 | `46.086 s` | `38.765 s` | `0.9830` | 983/1000 | `22 MB` |
| 64 | 32 | 25 | `74.053 s` | `23.881 s` | `0.9840` | 984/1000 | `22 MB` |
| 64 | 48 | 25 | `74.444 s` | `43.213 s` | `1.0000` | 1000/1000 | `22 MB` |

Latency loop:

| nlists | nprobe | width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 28 | 25 | `382.821 ms` | `414.706 ms` | `438.567 ms` | `368.566 ms` |
| 64 | 32 | 25 | `433.881 ms` | `453.544 ms` | `509.243 ms` | `418.803 ms` |
| 64 | 48 | 25 | `433.318 ms` | `452.825 ms` | `458.398 ms` | `425.895 ms` |

## Interpretation

At 25k rows, `nlists=64` does not improve the local full-recall frontier.
`64/48,width=25` reaches `1.0000` recall@10, but latency is effectively the
same as the simpler `32/32,width=25` point from packet 30044 and build time is
much higher.

Current 25k candidate ladder:

| point | recall@10 | p50 | p95 | build |
|---|---:|---:|---:|---:|
| `32/24,width=25` | `0.9760` | `331.674 ms` | `371.690 ms` | `46.138 s` |
| `32/28,width=25` | `0.9830` | `382.821 ms` | `414.706 ms` | `46.086 s` |
| `32/32,width=25` | `1.0000` | `434.858 ms` | `456.380 ms` | `45.068 s` |
| `64/48,width=25` | `1.0000` | `433.318 ms` | `452.825 ms` | `74.444 s` |

The simple 32-list full-probe point remains the best exact-oracle candidate for
this local 25k slice.

## Next Slice Recommendation

End this Task 28 slice with an IVF tuning summary/handoff packet. The local
recommendation should be:

- 10k inner-loop default: `nlists=32,nprobe=24,width=25` for fast high-recall,
  `32/32,width=25` for exact-oracle checks.
- 25k exact-oracle candidate: `nlists=32,nprobe=32,width=25`.
- Do not move to 50k local exact truth unless the exact baseline is reduced or
  cached externally; 25k/100 exact truth already took `18:46.749`.

## Artifacts

- `artifacts/pg18-ivf-anchor25k-routing-followup.sql`
- `artifacts/pg18-ivf-anchor25k-routing-followup.log`
- `artifacts/manifest.md`

## Validation

Packet-only change.

- `git diff --check`
