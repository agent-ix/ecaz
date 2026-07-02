# Task 120 Phase 6 Invariants

## Current Promotion State

Task 120 has not promoted a durable SPIRE coarse-rerank format or default.
The measured locations have these current decisions:

| Location | Current decision | Reason |
| --- | --- | --- |
| Local leaf coarse-rerank | Shelve for Task 120 | Packet 008 measured local leaf block pruning as a negative result; packets 010/011 show recall is bounded upstream by route/leaf selection, not final rerank width. |
| Topology refinement | Iterate, not promote | Packet 011 shows route overfetch recovers recall locally, but the cost is high and AWS 1M/distributed evidence is still owed before a default. |
| Distributed near-data rerank | Iterate, not promote | Packet 015 reached distributed read-ready state, but production-read shipping/merge metrics did not complete. |

Because no durable Task 120 format/default is promoted, the invariant contract
below is a gate for future work rather than a migration plan for existing
users.

## Universal Fallback Rule

Any durable coarse summary, rerank sidecar, or distributed near-data rerank
metadata used by SPIRE must be conservative under uncertainty:

- stale, missing, malformed, or version-skewed metadata may cause overfetch;
- it may fall back to exact/source scoring for the whole selected leaf or
  partition object;
- in strict distributed mode it may fail closed with an explicit blocker;
- in degraded distributed mode it may skip the affected remote worker or object
  only with explicit degraded diagnostics;
- it must never silently drop a candidate that would have survived without the
  stale or missing metadata.

## Local Leaf and Summary Invariants

| Event | Required invariant |
| --- | --- |
| Insert | Newly inserted rows must either update the relevant summary/sidecar before it can prune candidates, or force the affected leaf/object through exact/full-leaf behavior until a rebuild publishes fresh metadata. |
| Delete | Deleted rows must not remain eligible because of stale summary membership. If delete visibility is ambiguous, exact heap/source visibility wins and summary pruning must not hide a live replacement row. |
| Vacuum | VACUUM may reclaim obsolete heap/index payloads only after no active epoch/snapshot can depend on them. Summary/sidecar references to reclaimed row locations must be rejected or force overfetch. |
| Leaf split or movement | Summary identity must include enough epoch/object-version information to detect movement. Mixed old/new leaf metadata cannot be combined for pruning; fall back to full-leaf or fail closed. |
| Summary rebuild | Rebuilds must publish atomically as a new epoch/object version. Readers may use the old complete version or the new complete version, but not a partially rebuilt summary. |
| Malformed summary | Decode errors, dimension mismatch, non-finite values, row-count mismatch, or impossible rank/cap fields must disable pruning for that scope or fail closed. |

## Budget and Rerank Invariants

- Candidate caps and rerank widths are query-time policy, not correctness
  proof. If a cap binds before exact truth containment is proven for the
  current surface, it remains diagnostic-only.
- A coarse score may order or prioritize candidates, but exact/source rerank is
  the authority for final top-k whenever Task 120 claims recall.
- If a durable sidecar is missing for part of a selected leaf, that part must be
  exact/full-leaf scanned or merged with an overfetch path; it cannot be treated
  as an empty candidate source.
- Storage savings from summaries or sidecars do not justify promotion without
  recall, latency, containment, and row/byte-read evidence at the required
  scales.

## Distributed Near-Data Invariants

| Event | Required invariant |
| --- | --- |
| Remote worker version skew | Endpoint identity and wire-version checks must gate reads. Strict mode fails closed; degraded mode may skip only with explicit diagnostics and returned skip counters. |
| Stale remote placement | Requested epoch/object version must match published remote placement metadata. Mismatch must fail closed or degraded-skip; it must not dispatch against an unverified object. |
| Missing remote heap materialization | The AM path may return only coordinator-visible heap TIDs. Missing materialization is a blocker in strict mode and a diagnostic skip in degraded mode. |
| Remote timeout or cancellation | Timeout/cancel must surface through production-read counters and must not return partial unmarked candidates as if complete. |
| Worker-local exact rerank | Worker streams must identify how many candidates were generated, exact-reranked, shipped, merged, duplicated, and returned so coordinator recall/latency claims can be audited. |

## Final Closeout Dependency

These invariants satisfy Task 120 Phase 6 recordkeeping before promotion. They
do not close Task 120 by themselves. Closeout still requires a final packet that
uses completed Phase 5 shipping/merge evidence to recommend promote, iterate,
or shelve for local leaf, topology refinement, and distributed near-data rerank.
