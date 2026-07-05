# Review Request: C1 Task16 ADR-044 Storage-Policy Proposal

Current head at execution: `9c15918`

## Context

Packet `447` answered an important but incomplete question:

- forcing canonical `ecvector` inline (`STORAGE PLAIN`) makes the serious lane
  fast
- forcing it inline also makes small row rewrites materially heavier

Reviewer feedback on packet `447` was explicit that this is not enough to pick a
default. The missing work is:

- measure the other storage-policy cells, especially `EXTERNAL`
- spell out the alternative implementation path if heap storage modes do not
  yield a good default

ADR-043 now defers the storage-policy decision to ADR-044, so ADR-044 itself
needs to be visible on-branch and concrete enough to guide the remaining task-16
work.

## What this slice does

This docs-only slice adds `spec/adr/ADR-044-ecvector-rerank-source-location-and-storage-policy.md`
to the branch and makes two things explicit.

### 1. The measurement matrix is now written down as the actual decision gate

ADR-044 enumerates the remaining option space instead of implicitly assuming the
choice is just "`EXTENDED` vs `PLAIN`":

- heap storage modes:
  - `EXTENDED`
  - `EXTERNAL`
  - `MAIN`
  - `PLAIN`
- `PLAIN` mitigations:
  - `fillfactor`
  - structural vertical partitioning
- architectural alternatives:
  - C1 index-side cold-page rerank payload
  - C2 AM-owned sidecar relation
  - C3 custom TOAST strategy
- explicit "quality retreat" option:
  - quantized rerank only

It also records the must-measure cells before the default can be chosen:

- `EXTERNAL`
- `MAIN`
- `PLAIN + fillfactor`
- larger touched-column update probe
- detoast-vs-decompress decomposition when practical

### 2. C1 is now grounded in the real code seams, not just named abstractly

The new `Current-code fit for C1` section makes the important architectural
point explicit:

- current head already has hot/cold tuple separation
- both TurboQuant V3 and PqFastScan already point from a hot tuple to a cold
  rerank tuple via `reranktid`
- build already stages rerank tuples independently
- insert already writes rerank tuples through a dedicated path
- scan and vacuum already resolve rerank payload by index TID

So C1 is not "invent a second storage plane from scratch". It is an on-disk
format choice on top of an existing indirection seam.

The ADR now calls out the real fork in that implementation:

- widen `TqRerankTuple`, or
- add a sibling cold raw-f32 tuple kind

and explains why that composes naturally with ADR-042 native HNSW build.

## Why this matters

Task 16 was at risk of drifting into an implied decision:

- packet `447` measured a real tradeoff
- docs started reading as if the default answer was already "external for churn,
  plain for speed"

That was premature. This ADR puts the decision back behind the missing evidence
and gives the alternative implementation track a concrete shape.

That does two practical things:

1. reviewers can now argue about the actual decision matrix in one place
2. the next measurement slice has a stable target instead of ad hoc follow-up
   cells

## Validation

Docs-only slice. No tests rerun.

## Review focus

1. Is ADR-044 the right decision gate for task 16's remaining storage-policy
   work, or is a major option/cell missing?
2. Does the new C1 seam sketch accurately reflect current head:
   - hot/cold tuple separation already exists
   - build/insert/scan/vacuum already have the needed rerank-payload hook
3. Is the framing around `EXTERNAL` correct as the highest-value next
   measurement cell before heavier implementation work?
