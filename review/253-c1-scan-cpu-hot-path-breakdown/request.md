# Review Request: C1 Scan CPU Hot-Path Breakdown

## Context

Packet `252` landed a scan-local graph read cache in the ordered scan path and
reduced shared-buffer hits on the real `10k` scratch probe:

- `ef_search=40`: about `1505 -> 668` shared hits
- `ef_search=200`: about `6167 -> 2141` shared hits

That change was validated and pushed. However, the same scratch rerun still
shows large wall-clock cost:

- `ef_search=40`: about `126ms`
- `ef_search=200`: about `419ms`

So page rereads were not the whole C1 bottleneck.

## Problem

The remaining ordered-scan cost is now more likely CPU-side work in the scan
runtime itself:

- repeated graph traversal bookkeeping
- tuple decode / neighbor slicing
- scoring over candidate codes
- candidate frontier maintenance

The current explain counters do not break those costs down tightly enough to
justify the next optimization.

## Planned work

1. Add a narrow profiling seam for the ordered scan hot path, focused on CPU
   work rather than shared-buffer churn.
2. Measure where rescan time is going across:
   - candidate expansion
   - tuple decode / adjacency extraction
   - scoring
   - frontier/result maintenance
3. Use that evidence to pick the next optimization slice instead of guessing.
4. Keep the slice measurement-first:
   - no planner changes
   - no benchmark harness changes
   - no speculative algorithm rewrite without a measured target

## Exit criteria

- a pushed checkpoint records a concrete CPU-side breakdown for the real C1
  ordered scan path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy) if code
  changes are introduced
- the packet names the next optimization target using measured evidence
