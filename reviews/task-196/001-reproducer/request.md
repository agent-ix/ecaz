---
task: 196
packet: 001-reproducer
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 196 real-100k reproduction and attribution

This checkpoint reproduces the Task 193 stable-prefix failure on current
release code and identifies the root cause. The checked-in suite's explicit
`reject_multiple_windows` scenario fails production lazy10 after excluding the
first 40 ranked IDs:

```text
stable-prefix deepening re-requested 1 remote payloads
(window_start=20 window_end=30 window_remote_ids=7
 prefix_remote_rank_shifts=2 prefix_duplicate_ranked_ids=0)
```

The ranked window contains no duplicate IDs and begins exactly at the current
cursor, excluding duplicate traversal output and window overlap. Instead, two
already-materialized remote IDs moved raw rank when a deeper search re-sorted
equal exact distances. Payload reuse only inspected the old raw rank and took
that slot before verifying identity, so it discarded the reusable payload and
requested the immutable vec_id again.

The feature guard turns this into an error, but normal production silently
does the redundant request. Because the physical generation is immutable, the
impact is wasted owner work and a broken efficiency invariant, not result or
storage corruption.

## Task 191 lineage

The CustomScan implementation has no diff between Task 191's accepted merge
`f291bbb48` and Task 195's parent `adcd95623`. Task 191 ran this semantic matrix
only at 10k; its 100k performance matrix did not enable correctness drills.
This is therefore a deterministic 100k coverage hole in the accepted Task 191
implementation rather than a later regression.

## Proposed narrow fix

Reuse any already-materialized remote payload with the same immutable vec_id
within the previously proven prefix, regardless of its former raw rank. The
new search continues to define output order; traversal, visibility, failure,
payload, and BW×H behavior do not change. Packet 002 will carry the fix,
rank-shift unit coverage, and the full eight-scenario semantic rerun.

Please review the attribution and the identity-keyed reuse boundary. Exact
commands, binary identity, and retained evidence are in
`artifacts/manifest.md`.
