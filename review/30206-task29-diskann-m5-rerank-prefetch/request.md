# ec_diskann Apple-Silicon Heap-Block Prefetch — Negative Result

Reviewer: please review this Apple-Silicon-specific ec_diskann
checkpoint and its packet-local A/B measurement.

## Headline

The trial commit `e8c2ad76`
(`Prefetch ec_diskann heap rerank blocks`) was measured against the
post-`30205` head and **does not promote**. Every percentile moved
slightly the wrong way:

| metric | pre (NEON+heap-TID) | trial (+prefetch) | delta | rel |
|---|---:|---:|---:|---:|
| min | 14.15 ms | 14.4 ms | `+0.25 ms` | `+1.8%` |
| p50 | 14.8 ms | 15.0 ms | `+0.2 ms` | `+1.4%` |
| p95 | 15.45 ms | 15.6 ms | `+0.15 ms` | `+1.0%` |
| p99 | 16.8 ms | 16.85 ms | `+0.05 ms` | `+0.3%` |

The trial commit was reverted on the branch (`45557959`) so the
branch tip stays at the `30205` post state, which is the current
best ec_diskann Apple-Silicon checkpoint. The reverted commit is
preserved in history so reviewers can read the exact diff.

## Hypothesis

After `30205` landed the heap-TID sort, the rerank set is in
disk-friendly order but PG still serializes the per-row buffer
reads inside the rerank loop. Mirror the IVF prefetch in `3ef44426`:
issue a batched async prefetch over the sorted rerank set before
the rerank loop runs, so PG can start populating shared buffers
concurrently with the first few rerank rows.

The trial commit `e8c2ad76`:

- Adds a `prefetch: FnOnce(&[ItemPointer])` parameter to
  `scan::vamana_scan_with`, called once with the heap-TID-sorted
  rerank set after the sort and before the rerank loop.
- Wires a real `prefetch_heap_rerank_blocks` helper from
  `routine.rs::ec_diskann_amrescan` using the PG18
  `read_stream_begin_relation` API (or `PrefetchBuffer` on older
  PG), the same shape as the IVF helper.

Existing callers (the `vamana_scan` test shim and the index-build
path's call inside `routine.rs`) pass no-op prefetch closures, so
their semantics are unchanged.

## Result

Same fixture, same on-disk index, same `--force-index`, 200
iterations / pass, two passes per arm. Pre-arm passes come from
packet `30205` (since the pre code is identical to `30205`'s post);
trial-arm passes are this packet.

Pre arm (NEON + heap-TID, no prefetch — from `30205`):

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| pre 1 | 16.4 ms | 21.1 ms | 14.1 ms | 14.8 ms | 15.6 ms | 17.6 ms | 314.4 ms |
| pre 2 | 14.8 ms | 0.53 ms | 14.2 ms | 14.8 ms | 15.3 ms | 16.0 ms | 20.6 ms |

Trial arm (NEON + heap-TID + prefetch):

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| trial 1 | 16.6 ms | 20.9 ms | 14.5 ms | 15.0 ms | 15.7 ms | 18.0 ms | 310.8 ms |
| trial 2 | 15.0 ms | 0.50 ms | 14.3 ms | 15.0 ms | 15.5 ms | 15.7 ms | 20.4 ms |

(The `300+ ms` rows are autovacuum-shaped outliers; they inflate
`mean` / `max` / `stddev` but not the percentiles.)

The deltas are inside the per-pass stddev (`~0.5 ms`), but they are
**uniformly in the slow direction** across `min`, `p50`, `p95`, `p99`.
That is the opposite of the consistent same-direction win the
heap-TID sort produced in `30205`. Per the handoff bar — "treat
mixed or noisy results as negative unless they clearly promote" —
this is a non-promotion.

Recall is unchanged (the prefetch hook does not touch result
identity).

## Why prefetching does not help here

Two structural reasons:

1. **Warm cache.** The fixture is small enough that after a handful
   of iterations every rerank-eligible heap page is already in PG
   shared buffers. The prefetch hits cache and is a no-op for I/O —
   but it still pays buffer-table lookup, pin/unpin, and read-stream
   setup/teardown overhead.

2. **Two pins per block.** The trial implementation drains the
   read stream to completion before returning, which converts the
   intended "async prefetch" into a **synchronous preload**: every
   rerank block is pinned, returned, immediately released, and then
   re-pinned by the rerank loop's actual `fetch_heap_row_version`
   call. On warm cache that is pure overhead with no overlap
   benefit. (A correct async prefetch would hold the read stream
   open across the rerank loop and hand back buffers to it as it
   needs them, but that is a deeper restructure than this
   checkpoint was meant to be, and the warm-cache result above
   says it would not move main metrics on this fixture either way.)

So this is a real "the kernel is correct but the locality story
is exhausted at warm cache on this fixture" result, not a kernel
correctness issue.

## Recommendation

Do not land `e8c2ad76`. The revert (`45557959`) is on the branch.
The current best ec_diskann Apple-Silicon checkpoint remains the
post state of packet `30205` (NEON kernel + heap-TID-sorted rerank).

Two follow-on directions stay open and are NOT in scope here:

- **Cold-cache / IO-bound rerun.** The bench harness is warm. A
  cold-cache rerun on a corpus large enough that shared buffers
  cannot hold the whole heap could re-surface a prefetch win.
  Requires harness changes plus a much larger fixture.
- **Async-overlapping prefetch.** A correct async prefetch would
  hold the read stream open across the rerank loop and consume
  buffers from it as rerank rows are scored. Structurally a much
  bigger change to `scan::vamana_scan_with`; should not be tried
  before a cold-cache measurement justifies it.

The third packet-`30204` follow-on (exact rerank source decode
overhead) is also still open, with the caveat noted in `30205`
that the rerank path already borrows `&[f32]` rather than
allocating per row, so the source-decode lever is narrower than
naive intuition suggested. That one would need its own focused
investigation.

## Validation

- `cargo check --no-default-features --features pg18` (clean
  before and after the trial + revert).
- `cargo test --no-default-features --features pg18 --lib am::ec_diskann::scan`
  (29 tests pass; the trial added a no-op closure to the existing
  shim and changed nothing observable from the test suite, the
  revert restores the prior state).

## Artifacts

All artifacts live under `artifacts/`. See `artifacts/manifest.md`
for SHAs, commands, and full per-pass tables. Pre-arm tables are
reused from `review/30205-task29-diskann-m5-rerank-heap-order/artifacts/`.
