# Task 205 threshold/limit attribution disposition

## Finding

The bounded-L run proves that the production pushdown mechanism is active and
recall-neutral, but it does not support a threshold-only versus limit-only
latency or byte attribution.

The packet-local physical rows expose:

- `pushdown_rounds_with_threshold`, proving threshold activation;
- `neighbors_pruned`, which is emitted by the merged-batch truncation counter;
- response/request bytes, transport wait, and end-to-end latency; and
- recall, storage, topology, and owner-engagement rows.

They do not expose the number of neighbors removed by the threshold before the
merged-batch limit. The remote decode path sets the per-response
`neighbors_pruned` field to zero, while the stage counter records the aggregate
post-threshold merged-batch truncation. Therefore subtracting or relabeling
the existing counters would fabricate a split.

## What the evidence does establish

At L=32 versus L=4096, response bytes fall 52.1%, 60.8%, and 64.8% at 10k,
50k, and 100k; recall is identical at 0.9990, 0.9545, and 0.9275; and request
bytes are unchanged. Threshold rounds are nonzero for L=32 (6.46, 10.30,
9.78 per scan), while the L=4096 control has zero threshold rounds. The
aggregate `neighbors_pruned` rows are also nonzero for L=32 (163.58, 336.14,
341.90 per scan). These facts establish an active combined Algorithm 1
pushdown at L=32, not an independently quantified threshold contribution.

The end-to-end change remains modest (28.40 vs 29.90 ms at 10k, 39.00 vs
40.10 ms at 50k, and 37.30 vs 37.90 ms at 100k in the packet's table). The
packet therefore makes no claim that threshold pruning, batch limiting, or
either individual mechanism caused that movement.

## Disposition

The aggregate mechanism result is decision-usable for the next lanes with the
following restriction: Task 215 may use `candidate_heap_limit=32` as the
already-reviewed control behavior, but must not claim a threshold-only or
limit-only win. Task 216 must treat owner-side work as a separate attribution
lane and must not combine its candidate with Task 215's BW/H A/B.

Obtaining an exact threshold/limit split would require new instrumentation or
an isolated arm. That is outside this closeout's requested scope and is not
performed here. The missing split remains an explicit limitation for outside
review rather than a hidden inference.
