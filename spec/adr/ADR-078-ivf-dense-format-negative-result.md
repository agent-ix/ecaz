---
type: ADR
id: ADR-078
title: "IVF Dense Format Negative Result"
status: ACCEPTED
impact: Records that the page-spanning packed, columnar frozen-list, and page-scatter IVF dense-format investigations are not production formats; preserves page-local dense blocks, aligned dense blocks, coalescing, typed views, and coarse rerank as the Task 111 survivors.
date: 2026-06-18
---
# ADR-078: IVF Dense Format Negative Result

## Context

Task 111 tested several dense IVF posting layouts because row postings left too
little contiguous work for the scorer. The durable result is not that every
dense layout helped. The winning path is page-local dense posting blocks, the
aligned typed-view variant, scan-side coalescing, and the later
`coarse_rerank` contract built on that keeper path.

The abandoned lines were:

- A page-spanning packed posting format that fragmented one logical dense group
  across physical page segments.
- A columnar frozen-list format that moved list contents into grouped column
  payloads.
- A page-aware scatter scorer that tried to score columnar payloads without the
  contiguous copy fallback.

The evidence lives in the Task 111 review packets, especially
`reviews/task-111a/{004,007,008}` and `reviews/task-111c/{002,003,004,005}`.
Those packets show the same practical outcome: the extra formats either lost to
or failed to beat the simple contiguous-copy path. Copying a scorer-width group
into a contiguous scratch layout was cheaper than carrying a permanent
multi-segment or page-scattered storage format.

## Decision

Do not promote the page-spanning packed format, the columnar frozen-list format,
or the page-aware scatter scorer to the production IVF on-disk format.

Task 111f removes their tags, reloptions, scan paths, vacuum paths, build
writers, EXPLAIN counters, and tests before the Task 111 lane merges to `main`.
The surviving IVF dense formats are:

- Row postings for mutable and delta entries.
- Page-local dense posting blocks.
- Aligned dense posting blocks with typed-view decoding.
- Scan-side dense coalescing and typed-view controls.
- `coarse_rerank`, which remains tied to the surviving dense path.

Former experimental tag values are reserved historical values. They must not be
silently reused for a different layout.

## Consequences

The IVF codebase carries fewer default-off branches and fewer compatibility
liabilities. Future dense-format work must start from fresh evidence and an
explicit format decision instead of reviving the removed layouts by inertia.

The negative result is still valuable: it records that zero-copy is not
automatically better when the scorer wants contiguous, scorer-width batches.
For this access method, locality and simple contiguous group assembly beat the
more complex page-spanning and page-scattered alternatives measured in Task 111.
