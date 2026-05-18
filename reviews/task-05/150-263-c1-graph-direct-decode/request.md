# Review Request: C1 Graph Direct Decode

## Context

Packet `262` removed the temporary full-tuple byte copy in `src/am/graph.rs`
and produced a small but consistent warm win on the verified `10K`,
`m=8`, `ef_search=40` surface.

Current warm read still remains far above the C1 target:

- baseline before packet `262`: `p50=14.315ms`, `mean=14.194ms`
- after packet `262`: roughly `p50=13.9-14.0ms`, `mean=14.0ms`

So C1 still needs larger hot-path reductions than the tuple-byte copy cut.

## Problem

`src/am/graph.rs` still decodes through the generic page tuple structs:

- `load_graph_element(...)` decodes a `page::TqElementTuple`, then moves fields
  into `GraphElement`
- `load_graph_neighbors(...)` decodes a `page::TqNeighborTuple`, then moves
  fields into `GraphNeighbors`

That leaves extra owned decode churn on the hot path:

- `TqElementTuple::decode(...)` still builds an intermediate heap-tid `Vec`
  and then collects into another `Vec`
- neighbor decode still materializes the generic tuple struct before the cache
  structs take ownership

## Planned work

1. Decode element tuples directly into `GraphElement` in `src/am/graph.rs`.
2. Decode neighbor tuples directly into `GraphNeighbors` in `src/am/graph.rs`.
3. Keep the tuple-layout validation identical to the existing page decoders.
4. Re-run the full checkpoint gate and the verified warm per-cell `10K`
   `m=8`, `ef_search=40` surface.

## Current draft

Implemented probe:

- added direct tuple decoders in `src/am/graph.rs` so graph cache loads no
  longer round-trip through `page::TqElementTuple` / `page::TqNeighborTuple`
- added graph-level layout tests to keep the direct decode path aligned with
  the existing encoded page format

Validation read:

- `cargo test`: green
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: green
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`:
  green

Warm measurement against the packet `262` read:

- packet `262` warm reads landed around `p50=13.9-14.0ms`,
  `p95=16.1-16.2ms`, `mean=14.0ms`
- direct-decode probe: `p50=14.312ms`, `p95=19.469ms`, `p99=22.118ms`,
  `mean=14.851ms`

Current read:

- this probe regressed the warm steady-state surface and should be discarded
- the direct decode path may have worsened instruction/cache behavior enough
  to swamp the allocation savings on this fixture
- next work should back out this code and move to a different seam rather than
  commit it

## Exit criteria

- graph tuple loads no longer round-trip through generic page tuple structs
- the change is validated through `cargo test`,
  `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`, and
  `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the verified warm per-cell surface is rerun and compared to the packet `262`
  read
