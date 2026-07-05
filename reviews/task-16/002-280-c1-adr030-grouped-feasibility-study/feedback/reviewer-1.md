## Feedback: ADR-030 Grouped Feasibility Study

This is the packet that decided the lane was worth continuing. The v2 follow-on
packets (310-328) executed the architecture this study justified.

### What held up

The 280 study result is consistent with what packet 311 re-measured under a tighter
harness: grouped PQ4 on SRHT-transformed vectors is accurate enough at `group_size =
16` to serve as an approximate scorer, with a large arithmetic speedup headroom over
exact scoring.

### What the 310-328 lane did with this result

- Turned "it's accurate enough" into a versioned index format (packet 312).
- Decided to split hot (traversal) from cold (rerank) so the approximate scorer never
  touches rerank bytes during traversal (packet 313).
- Built the write lane behind a double gate (env var + source-column), with runtime
  still rejecting v2 explicitly (packets 315-323).
- Added read-side seams for the eventual scorer (packets 324-328) without enabling
  it, leaving scoring as a narrow next packet.

### Thing to keep validating

The feasibility study is on an in-process study harness. Before the gate is lifted,
re-measure under real pgrx scan conditions against the composed binary+grouped+rerank
pipeline, including recall at typical `ef_search` values. The speedup multiplier and
the rank quality in that setting are the numbers that matter for the gate-lifting
decision.
