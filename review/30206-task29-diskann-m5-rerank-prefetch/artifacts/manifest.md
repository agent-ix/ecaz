# Artifact Manifest

Packet: `review/30206-task29-diskann-m5-rerank-prefetch`

Lane: ec_diskann Apple-Silicon rerank heap-block prefetch A/B —
**negative result**.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`), local PG18 18.3 (Homebrew),
socket `/Users/peter/.pgrx`, port `28818`.

Surface: kernel-stress lane on the existing real-data prefix
`m5_diskann_real10k_w800` (built and benchmarked in packet `30204`,
reused unchanged in `30205` and here). 200 iterations / pass,
`L=800`, `rerank_budget=800`, `--force-index`, warm cache.

## Code SHAs

- pre (NEON + heap-TID-ordered fetch, no prefetch): `4154fcb6` =
  `e191a9e1` code state, post-`30205`. Installed binary sha256
  `20d6c4e2d2c9839bddd334f61c6f139147a71ec4d3f12e0a35400f7646509cd4`.
  Pre-arm numbers come from `30205`'s post passes (already
  packet-local under `review/30205-.../artifacts/`).
- trial (prefetch on top of pre): `e8c2ad76`
  (`Prefetch ec_diskann heap rerank blocks`). Installed binary sha256
  `92173c164fde9ae56799f9235033b069144459d467f7d286aeb4caf059c94663`.
- revert: `45557959`
  (`Revert "Prefetch ec_diskann heap rerank blocks"`) brings the
  branch tip back to the `4154fcb6` code state. The reverted commit
  is preserved in history so reviewers can read the exact diff.

## Hypothesis

After packet `30205` landed the heap-TID sort, the rerank set is in
disk-friendly order but PG still serializes the per-row buffer reads
inside the rerank loop. Mirror the IVF prefetch in commit `3ef44426`:
issue a batched async prefetch over the sorted rerank set before the
rerank loop runs, so PG can start populating shared buffers
concurrently with the first few rerank rows.

The trial commit `e8c2ad76` adds a `prefetch` closure parameter to
`scan::vamana_scan_with` and wires a real `prefetch_heap_rerank_blocks`
helper from the SQL scan path. The helper uses the PG18
`read_stream_*` API (or `PrefetchBuffer` on older PG), the same shape
as the IVF helper.

## Result — does not promote

200 iterations / pass on the same on-disk
`m5_diskann_real10k_w800` index.

Pre arm (NEON + heap-TID, **no** prefetch — from `30205`):

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| pre pass 1 (`30205/artifacts/latency-post-table.log`) | 16.4 ms | 21.1 ms | 14.1 ms | 14.8 ms | 15.6 ms | 17.6 ms | 314.4 ms |
| pre pass 2 (`30205/artifacts/latency-post-confirm-table.log`) | 14.8 ms | 0.53 ms | 14.2 ms | 14.8 ms | 15.3 ms | 16.0 ms | 20.6 ms |

Trial arm (NEON + heap-TID + prefetch):

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| trial pass 1 (`latency-post-table.log`) | 16.6 ms | 20.9 ms | 14.5 ms | 15.0 ms | 15.7 ms | 18.0 ms | 310.8 ms |
| trial pass 2 (`latency-post-confirm-table.log`) | 15.0 ms | 0.50 ms | 14.3 ms | 15.0 ms | 15.5 ms | 15.7 ms | 20.4 ms |

(One autovacuum-shaped `300+ ms` outlier in each pass-1 row inflates
`mean` / `max` / `stddev` but not the percentile columns.)

Pass-averaged percentile deltas (trial minus pre):

| metric | pre avg | trial avg | delta | rel |
|---|---:|---:|---:|---:|
| min | 14.15 ms | 14.4 ms | `+0.25 ms` | `+1.8%` |
| p50 | 14.8 ms | 15.0 ms | `+0.2 ms` | `+1.4%` |
| p95 | 15.45 ms | 15.6 ms | `+0.15 ms` | `+1.0%` |
| p99 | 16.8 ms | 16.85 ms | `+0.05 ms` | `+0.3%` |

Every metric moved **slightly the wrong way**. The deltas are
within the per-pass stddev (`0.5 ms`) but they are uniformly in the
slow direction across `min`, `p50`, `p95`, `p99`. That is the
opposite of the consistent same-direction win the heap-TID sort
produced in `30205`.

## Why prefetching does not help on this workload

Two structural reasons, in order of importance:

1. **Warm cache.** The `m5_diskann_real10k_w800` corpus is tiny
   (10000 rows x ~6 KiB ecvector + small extras), so after a few
   iterations of the 200-iteration sweep, every rerank-eligible heap
   page is already in PG shared buffers. `pg_sys::PrefetchBuffer` /
   `read_stream_begin_relation` finds the page in cache and is a
   no-op for actual I/O — but it still pays buffer-table lookup,
   pin/unpin, and (for the `read_stream` path) read-stream setup
   and teardown overhead.

2. **The trial implementation pins every block twice.** The IVF
   helper, which this implementation copied, drains the read stream
   to completion before returning; that converts an "async
   prefetch" into a synchronous preload that pins, returns, and
   immediately releases every rerank block. The subsequent rerank
   loop then re-pins each block when it does the actual heap fetch.
   On warm cache that is pure overhead with no overlap benefit. To
   actually overlap I/O with rerank work the read stream would need
   to stay open across the rerank loop and hand back buffers to it,
   which is a deeper restructure than the narrow checkpoint this
   slice was meant to be.

So this is a real Apple-Silicon "the kernel is correct but the
locality story is exhausted at warm cache on this fixture" result,
not a kernel correctness issue.

## Recommendation

Do not land `e8c2ad76`. The branch tip after `45557959` reverts
back to the `30205` post state, which is the current best
ec_diskann Apple-Silicon checkpoint.

Two specific follow-on directions remain, both deferred to a future
slice:

- **Cold-cache / IO-bound rerun.** The current bench harness runs
  warm. A cold-cache rerun (`DISCARD ALL` + cluster restart between
  queries, or a corpus large enough that shared buffers cannot hold
  the whole heap) might re-surface a real prefetch win. That is
  measurement-driven, requires harness changes, and is out of scope
  for this packet.
- **Async-overlapping prefetch.** A correct async prefetch would
  hold the read stream open across the rerank loop and consume
  buffers from it as rerank rows are scored, rather than draining
  the stream up front. That is structurally a much bigger change
  to the rerank loop in `scan::vamana_scan_with` and should not be
  attempted before a cold-cache measurement justifies it.

The remaining packet-`30204` follow-on (exact rerank source decode
overhead) is also still open, with the caveat noted in `30205` that
a first read of `with_heap_source_vector` already shows the rerank
path borrowing `&[f32]` rather than allocating a per-row `Vec<f32>`,
so the source-decode lever is narrower than naive intuition
suggested.

## Commands

```
ecaz --log-file artifacts/install-pg18-post.log dev install ecaz-pg-test --pg 18

ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file artifacts/latency-post-cli.log \
  bench latency --prefix m5_diskann_real10k_w800 --profile ec_diskann \
  --k 10 --sweep 800 --iterations 200 --concurrency 1 \
  --force-index --sample-backend-memory \
  --log-output artifacts/latency-post-table.log

ecaz ... --log-file artifacts/latency-post-confirm-cli.log \
  bench latency ... --log-output artifacts/latency-post-confirm-table.log
```

Pre-arm commands and tables are reused from
`review/30205-task29-diskann-m5-rerank-heap-order/artifacts/`.

## Artifact list

- `manifest.md`
- `install-pg18-post.log`
- `latency-post-table.log`, `latency-post-cli.log`
- `latency-post-confirm-table.log`, `latency-post-confirm-cli.log`
