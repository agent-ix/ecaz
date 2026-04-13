# Review Request: C1 ADR-031 Tier 2 Pin-and-Hold Borrowed Score

## Context

Packet `289` cleared ADR-031 itself as the cause of the higher-`ef_search`
quality shift. The current Tier 1 ADR-031 path remains the best warm-latency
surface we have on the real `50k` seam, with the canonical `m=8`,
`ef_search=40` warm run around:

- `p50 ~= 1.48ms`
- `p99 ~= 2.4ms`
- `mean ~= 1.51ms`

Reviewer feedback on the ADR-031 arc identified one remaining hot-path copy on
the exact-score miss path:

- `LoadedElementScoreInput` still materializes `element.code.to_vec()`
- that copy exists because exact scoring happens after the page buffer is
  released

## Problem

Tier 1 eliminated most scan-cache churn, but the ADR-031 hot path still copies
the quantized code payload into an owned `Vec<u8>` before exact scoring can run
on newly loaded elements.

That leaves a clear next seam:

- hold the graph element buffer pinned while exact scoring happens
- score directly from borrowed `TqElementTupleRef.code`
- delete the remaining `element.code.to_vec()` copy from the ADR-031 exact
  score path

## Planned Slice

Implement the Tier 2 pin-and-hold path described in the review feedback:

1. split graph element reads into a pin-and-hold API in `src/am/graph.rs`
2. move exact scoring for newly loaded elements into that pin scope
3. score directly from borrowed tuple bytes instead of an owned copied buffer
4. keep the rest of the ADR-031 cache shape unchanged unless the pin scope
   forces a small supporting change

## Success Criteria

- exact scoring on the ADR-031 miss path no longer requires
  `element.code.to_vec()`
- the code remains correct under PostgreSQL buffer lifetime rules
- `cargo test`, `cargo pgrx test pg17`, and clippy are green
- the packet records whether the Tier 2 seam materially improves the canonical
  warm real-`50k` ADR-031 surface
