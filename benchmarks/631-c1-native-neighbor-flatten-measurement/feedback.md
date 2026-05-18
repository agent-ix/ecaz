# Feedback: 631 Native Neighbor Flatten Measurement

## Verdict: Accept

0.9–2.3% improvement is small but the change is correct. Replacing per-node
`HashSet` allocation with small-vector linear dedup in `flatten_native_neighbor_slots`
is the right call: neighbor counts are bounded by `m * layers` (typically ≤ 30),
making linear scan cheaper than hash table construction and teardown at that size.
The request correctly characterizes this as an allocation cleanup, not a
algorithmic speedup.

## One Clarification

This is NOT the `BuildState::push` dedup path. `flatten_native_neighbor_slots`
operates on per-node neighbor slot lists (small N bounded by m). The O(N²)
concern from packet 626 was about BuildState::push, which uses a HashMap (no
linear scan). These are separate paths. The current change is correct and the
size-bounded linear dedup is appropriate here.

## Closure

This closes the low-risk serial cleanup lane. The 27.5s residual graph phase
requires graph-assembly parallelization, not further per-node micro-optimization.

## No Issues
