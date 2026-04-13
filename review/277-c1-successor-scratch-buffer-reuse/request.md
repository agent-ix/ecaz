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

## Planned work

1. Add scan-local reusable scratch vectors for successor/result staging.
2. Rewire the hot scan traversal path to clear and reuse those buffers instead
   of allocating a fresh `Vec` on each expansion.
3. Keep graph traversal semantics unchanged.
4. Re-run the checkpoint gate and the verified warm real-corpus cell.
5. Record whether the slice is worth keeping or is another low-signal discard.

## Exit criteria

- the hot scan successor path reuses scan-local scratch buffers instead of
  allocating fresh temporary vectors on each expansion
- `cargo test` is green
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` is green
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  is green
- the packet records the verified warm real-corpus before/after readout
