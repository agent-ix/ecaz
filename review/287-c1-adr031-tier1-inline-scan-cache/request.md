# Review Request: C1 ADR-031 Tier 1 Inline Scan Cache

## Context

Packets `281` through `286` established that ADR-031 is a real keep:

- cached ADR-031 clears `NFR-001` on the real `50k` lane
- runtime recall matches exact quantized results at the target seam
- persisted binary sidecars are worth keeping for cold startup

Reviewer feedback on packets `281` and `285` narrowed the next warm-path work
to two follow-ups:

1. Tier 1: replace scan-local `Vec<u64>` / `Vec<ItemPointer>` storage with
   bounded inline storage on the cached graph-element path
2. Tier 2: pin-and-hold graph-element reads so exact scoring can borrow code
   bytes directly instead of copying `element.code.to_vec()`

This packet is only about Tier 1. Tier 2 is a larger pin-lifetime refactor and
should stay separate.

## Problem

The ADR-031 hot path still allocates per cached graph element in
[src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs):

- `CachedGraphElement.heaptids: Vec<ItemPointer>`
- `CachedGraphElement.binary_words: Vec<u64>`

That shape is overkill for bounded payloads:

- heap tids are already capped by
  [page::HEAPTID_INLINE_CAPACITY](/home/peter/dev/tqvector/src/am/page.rs)
- the real ADR-031 target seam uses `1536` dimensions, so binary-sign codes are
  `24` `u64` words

If those two vectors are replaced with inline storage on the scan-local cache
path, we should remove some per-element allocator churn without touching the
larger buffer-lifetime boundary.

## Planned Slice

In [src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs):

- replace cached heap tids with inline `[ItemPointer; 10] + count`
- replace cached binary words with an inline-first representation sized for the
  ADR-031 target seam, with a safe fallback if a wider code path appears
- update the result-materialization path so it reuses the inline cached heap
  tids instead of rebuilding temporary `Vec`s where possible

## Success Criteria

- no behavior change in scan results
- all usual gates green
- a warm verified real-corpus read on the ADR-031 canonical seam records
  whether the inline cache shape moves latency enough to keep

## Implementation Checkpoint

The Tier 1 inline cache slice is now implemented in
[src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs).

What landed:

- `CachedGraphElement.heaptids` no longer uses `Vec<ItemPointer>`; it now uses
  bounded inline storage sized to
  [page::HEAPTID_INLINE_CAPACITY](/home/peter/dev/tqvector/src/am/page.rs)
- `CachedGraphElement.binary_words` no longer uses `Vec<u64>` on the ADR-031
  target seam; it now uses inline-first storage sized for the `1536`-dim
  binary-sign payload (`24` words) with a heap fallback for wider future paths
- cached graph-result materialization now writes directly from the cached inline
  heap-tid slice into `ScanResultState`, instead of rebuilding a temporary
  `Vec<ItemPointer>` on the hot path

What did **not** land:

- no pin-and-hold buffer lifetime changes
- no borrowed exact-score path yet
- no page-layout or tuple-format changes

So this packet stays in the intended Tier 1 scope: scan-local cache shape and
immediate result materialization only.

## Validation

Green gate on the code checkpoint:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The slice also adds narrow scan-local tests for:

- inline cached heap-tid storage
- inline cached binary-word storage at the ADR-031 target width
- safe heap fallback when a wider binary-word path appears

## Warm Measurement

Release benchmark command on the canonical ADR-031 seam:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output /tmp/adr031_tier1_inline_scan_cache_real_50k_m8_ef40_warm.summary
```

Observed canonical read:

```text
m=8
ef_search=40
n=1000
p50=1.480ms
p95=2.084ms
p99=2.390ms
mean=1.507ms
server_qps=663.72
wall=14.04s
```

Immediate reconfirm on the same seam:

```text
m=8
ef_search=40
n=1000
p50=1.485ms
p95=2.047ms
p99=2.422ms
mean=1.510ms
server_qps=662.30
wall=13.75s
```

Reference point from packet `282` before this slice:

- `p50=4.633ms`
- `p99=7.661ms`
- `mean=4.727ms`

## Readout

Tier 1 is a decisive keep.

Against the standing ADR-031 warm canonical baseline, the inline scan-cache
slice improves the real `50k`, `m=8`, `ef_search=40` seam by about:

- `4.727ms -> 1.507ms` mean (`~68%` lower latency)
- `4.633ms -> 1.480ms` p50 (`~68%` lower latency)
- `7.661ms -> 2.390ms` p99 (`~69%` lower latency)

That is much larger than the original Tier 1 expectation. The most likely
reason is that the old cached path was still paying allocator and temporary-copy
costs more often than the earlier hot-path profile made obvious:

- cached element heap tids no longer allocate
- persisted ADR-031 sidecars no longer need to survive as `Vec<u64>` in the
  scan-local cache on the target seam
- cached graph-result materialization no longer rebuilds a throwaway heap-tid
  vector before draining duplicates

The repeated canonical rerun makes the result look real rather than lucky.

It also means the real warm `50k` ADR-031 seam is now below the user-stated
`<2ms` target.

## Next Step

Tier 2 is still available:

1. split graph-element reads into a pin-and-hold boundary
2. exact-score directly from borrowed `TqElementTupleRef.code`
3. remove the remaining `element.code.to_vec()` copy in the binary path

But that is no longer required to make ADR-031 look credible. It is now an
optional follow-on if we want to push cold path, tail latency, or headroom even
further before branching into ADR-032.
