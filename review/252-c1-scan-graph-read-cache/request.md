# Review Request: C1 Scan Graph Read Cache

## Context

Packet `251` established the first real C1 runtime profile:

- warm-cache SQL execution at `m=8, ef_search=40` is already about `6.8ms`
- the ordered scan remains front-loaded in `amrescan`
- tuple emission is negligible compared to rescan/setup
- the real query shape still touches far more shared buffers than the current
  element-only counters report:
  - about `1505` shared-buffer hits at `ef_search=40`
  - about `6167` shared-buffer hits at `ef_search=200`

So the next optimization target is page-touch volume during graph search, not
the visible tuple-emission path.

## Problem

The current graph read surface repeatedly calls:

- `graph::load_graph_element(...)`
- `graph::load_graph_neighbors(...)`
- `graph::load_graph_adjacency(...)`

Those helpers read and decode tuples afresh each time. During one ordered scan,
the same element and neighbor tuples can be revisited multiple times across:

- upper-layer descent
- layer-0 seed search
- later frontier/result materialization

That repeated tuple reread / redecode pattern is the leading C1 suspect after
packet `251`.

## Planned work

1. Add a scan-local cache for graph reads in `TqScanOpaque`.
2. Route ordered scan search/materialization through cached graph-element /
   adjacency access instead of unconditional rereads.
3. Keep the slice narrow:
   - no planner changes
   - no benchmark harness changes
   - no speculative executor changes outside the graph read path
4. Re-run the profile helper plus representative `EXPLAIN (ANALYZE, BUFFERS)`
   probes to verify buffer-hit reduction before claiming improvement.

## Exit criteria

- a pushed checkpoint reduces repeated graph page access on the current real
  `10k` C1 path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records before/after profile evidence, not just code intent
