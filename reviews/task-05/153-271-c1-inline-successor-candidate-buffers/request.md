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

## Outcome

Discarded.

The implementation replaced the transient per-expansion successor `Vec` with an
inline `SmallVec<[BeamCandidate; 16]>` in the hot graph and scan successor
helpers, and generalized the local successor-closure seams so those helpers no
longer required `Vec` specifically.

The code itself was correct after validation, but it did not buy a trustworthy
latency win on the canonical warm C1 seam, so the code was reverted and nothing
from this probe is kept in `main`.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

An earlier full `cargo pgrx test pg17` attempt during development reported a
single failure on `test_tqhnsw_rescan_builds_bootstrap_candidate_frontier`, but
that did not reproduce on a targeted rerun or on the later full green sweep.

## Measurements

Canonical comparison point from packet `270`:

- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`,
  `session-mode=per-cell`, `timing-mode=cached-plan`
- baseline: `p50=10.753ms`, `p95=12.784ms`, `p99=14.034ms`, `mean=10.720ms`

First rerun on this packet's build:

- `p50=11.094ms`, `p95=13.662ms`, `p99=17.206ms`, `mean=3.636ms`,
  `min=-799.355ms`, `max=19.679ms`
- discarded as invalid because the server-side `clock_timestamp()` timing path
  produced a negative per-query duration on WSL2, which poisoned the mean

Two valid reruns on the same seam:

- rerun 1: `p50=10.990ms`, `p95=12.943ms`, `p99=14.750ms`, `mean=10.969ms`
- rerun 2: `p50=10.828ms`, `p95=12.777ms`, `p99=13.694ms`, `mean=10.799ms`

Those valid reads are effectively flat to slightly worse than packet `270`, and
not strong enough to justify the added type complexity and dependency.

## Decision

- revert the inline successor buffer implementation
- keep packet `271` as a recorded failed experiment
- do not land `smallvec` or the generalized successor-return-type changes

## Exit criteria

- hot per-expansion successor lists no longer require heap `Vec` allocation in
  the common `m=8` path
- existing graph/search/scan tests remain green
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- warm verified `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`,
  `cached-plan` read recorded
