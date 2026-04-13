# Review Request: C1 Successor Scratch Buffer Reuse

## Context

Packet `276` tried the smaller inline-heap-tid decode ownership change and was
discarded. The code stayed mechanically small, but under the full `cargo test`
suite it introduced order-dependent bootstrap-frontier regressions, so it was
not safe to keep for a minor allocation win.

The broader direction is still valid: warm C1 is no longer blocked on planner
or scoring correctness, but it remains materially above target because the scan
path still spends real time on repeated allocation and decode churn around graph
successor expansion.

Current verified warm steady-state on real `10K`, `m=8`, `ef_search=40`,
`warm-after-prime3`, `per-cell`, `cached-plan` is still anchored by packet
`270`:

- `p50=10.753ms`
- `p95=12.784ms`
- `p99=14.034ms`
- `mean=10.720ms`

## Problem

The graph traversal path still allocates fresh temporary vectors while walking
successors:

- `cached_scan_successor_candidates_for_layer(...)` in `src/am/scan.rs` builds a
  new `Vec<BeamCandidate<_>>` on every expansion
- the visible-seed expansion path in `src/am/graph.rs` still collects temporary
  seed/result vectors
- these allocations happen on the warm steady-state path even when the graph and
  score caches are already populated

This is a narrower seam than another tuple ownership rewrite, and it does not
need to change bootstrap result-state semantics.

## Implementation

Discarded.

The original draft targeted scan-local reusable scratch vectors, but the actual
bounded probe tightened that idea to a smaller stack-buffer seam:

1. generalize the graph successor helpers so closures can return any
   `IntoIterator<Item = BeamCandidate<_>>` rather than an owned `Vec`
2. use a stack-backed successor candidate buffer on the scan-local cached graph
   path so common `m=8/16` cases can avoid heap allocation for successor lists
3. keep graph traversal behavior and ordering rules unchanged

This kept the slice narrow and avoided another bootstrap result-state ownership
change.

## Result

Discarded.

The code checkpoint validated cleanly:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

One intermediate `cargo pgrx test pg17` run failed only because I mistakenly
ran it in parallel with `cargo test`, which collided on the shared pg-test data
directory. The sequential rerun was green.

The warm verified real-corpus read regressed relative to the packet `270`
baseline:

```text
baseline (packet 270):
  p50=10.753ms p95=12.784ms p99=14.034ms mean=10.720ms

probe run 1:
  p50=11.136ms p95=12.934ms p99=14.686ms mean=11.064ms

probe run 2:
  p50=11.401ms p95=15.520ms p99=18.323ms mean=11.772ms
```

So the stack-buffer candidate path was flat-to-worse on both confirmation runs.
That is not enough signal to keep another allocation tweak in the branch.

## Conclusion

This slice is not worth keeping. The remaining C1 allocation wins are either
smaller than expected or offset by extra generic/iterator overhead when applied
at this seam. The next move should return to a more promising direction:

- either a different zero-copy decode seam with clearer upside
- or back to `ADR-029` at a better runtime insertion point than packet `275`

## Exit criteria

- the packet records the keep/discard outcome and the verified warm
  real-corpus before/after readout
