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
