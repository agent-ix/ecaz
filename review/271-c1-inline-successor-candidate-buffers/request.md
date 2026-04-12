# Review Request: C1 Inline Successor Candidate Buffers

## Context

Packet `270` kept the hot scan/graph successor path on the same allocation-cut
track and produced another real warm win:

- before: `p50=10.932ms`, `mean=10.993ms`
- after: `p50=10.753ms`, `mean=10.720ms`

That result says the transient graph traversal buffers are still worth chasing.

## Problem

Even after removing the temporary neighbor-tid `Vec`, the hot successor-loading
paths still materialize a fresh heap `Vec<BeamCandidate<...>>` on every graph
expansion:

- layer-0 successors are capped at `2m`
- upper-layer successors are capped at `m`
- on the C1 target lane (`m=8`), those transient lists are at most `16`
  candidates and usually smaller

So the common case should fit comfortably in an inline buffer instead of taking
a heap allocation for every expansion.

## Planned work

1. Introduce an inline successor buffer type for the hot graph/scan successor
   helpers.
2. Generalize the local graph-search helpers so successor closures can return an
   inline buffer instead of requiring `Vec`.
3. Keep the externally visible ordered-result surfaces as `Vec` where they
   represent real result sets rather than transient per-expansion scratch.
4. Rerun the full checkpoint gate and the warm verified `10K`, `m=8`,
   `ef_search=40`, `warm-after-prime3`, `per-cell`, `cached-plan` seam.

## Exit criteria

- hot per-expansion successor lists no longer require heap `Vec` allocation in
  the common `m=8` path
- existing graph/search/scan tests remain green
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`,
  `cached-plan` read recorded
