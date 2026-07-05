# Task 28 IVF Current Auto Recommendation

This packet records the current recommendation for IVF `storage_format =
'auto'` based on the latest local measurements. It is a synthesis packet over
prior packet-local artifacts, not a new measurement run.

This does not change the code default. The task file says A10 records the
recommendation and any default change should be handled separately.

## Source Packets

- Packet 30084: first 10k TurboQuant / PQ-FastScan / RaBitQ head-to-head
  smoke.
- Packet 30091: 100k PQ-FastScan g8 versus TurboQuant comparison at
  `nlists=64`.
- Packet 30094: 100k PQ-FastScan g8 `nlists=128`, `rerank_width=500`
  nprobe middle sweep.
- Packet 30095: 100k PQ-FastScan g8 `nlists=256`, `rerank_width=500`
  sweep.

## Current Evidence

The early 10k packet 30084 favored TurboQuant because PQ-FastScan used the
initial group-size/default shape and had unacceptable recall at
`rerank_width=25`. It was still much smaller and faster, which made recall
recovery the right follow-up.

After the grouped-PQ work, the 100k evidence flipped:

| profile | surface | recall@10 | p50 | p95 | p99 | index size |
|---|---|---:|---:|---:|---:|---:|
| TurboQuant | n64/p32/w25 | 0.9930 | 464.8 ms | 538.0 ms | 556.8 ms | 87 MB |
| PQ-FastScan g8 | n64/p32/w750 | 0.9930 | 279.5 ms | 312.5 ms | 323.1 ms | 18 MB |
| PQ-FastScan g8 | n128/p48/w500 | 0.9920 | 238.5 ms | 262.1 ms | 274.7 ms | 19 MB |
| PQ-FastScan g8 | n128/p64/w500 | 0.9940 | 295.8 ms | 322.5 ms | 332.8 ms | 19 MB |
| PQ-FastScan g8 | n256/p96/w500 | 0.9940 | 270.1 ms | 305.7 ms | 342.2 ms | 20 MB |

RaBitQ is selectable and correctness-tested, but packet 30084 showed the
current IVF RaBitQ scan path is not latency-competitive yet: it matched
TurboQuant recall at 10k, but took p50 `1276.7 ms` for a narrowed nprobe-32
latency smoke. That makes RaBitQ a future optimization path, not the current
default candidate.

## Recommendation

The current measured default recommendation is PQ-FastScan g8 for high-dim
real-corpus IVF once a default-change task is opened. The best measured local
profiles are:

- Low-latency high-recall: `nlists=128`, `nprobe=48`, `pq_group_size=8`,
  `rerank_width=500`, recall@10 `0.9920`, p50 `238.5 ms`.
- Quality-biased: `nlists=256`, `nprobe=96`, `pq_group_size=8`,
  `rerank_width=500`, recall@10 `0.9940`, p50 `270.1 ms`.

Do not change `storage_format = 'auto'` in this branch solely from this
synthesis packet. The next code slice should either:

1. Add an explicit follow-up task to change `auto` to PQ-FastScan g8 for
   high-dimensional IVF, or
2. Keep `auto` as TurboQuant but document that users should select
   `storage_format = 'pq_fastscan', pq_group_size = 8` for the current
   measured 100k high-recall lane.

The second option is more conservative until the 10k and 25k A10 comparison
is re-run with `pq_group_size=8` and the wider rerank frontier.

## Remaining A10 Gap

A full A10 closure still needs:

- Re-run 10k and 25k head-to-head with PQ-FastScan g8 rather than the initial
  PQ-FastScan shape.
- Include recall@100, memory high-water mark, and cold/warm cache state.
- Decide whether `auto` should be dimension-sensitive, corpus-size-sensitive,
  or remain a stable global default with documentation.

## Artifacts

See `artifacts/manifest.md` for source-packet pointers.
