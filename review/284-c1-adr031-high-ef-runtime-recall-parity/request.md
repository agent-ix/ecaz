# Review Request: C1 ADR-031 Higher-EF Runtime Recall Parity

## Context

Packet `282` showed that the cached ADR-031 runtime path clears `NFR-001` on
the normative real `50k` lane at:

- `m=8`
- `ef_search=40`

Packet `283` then showed that the same live graph path matches the exact
quantized top-10 outputs on the full `1000`-query real `50k` table at that same
`m=8`, `ef_search=40` point:

- `graph_recall_at_10 = 0.8397`
- `exact_quantized_recall_at_10 = 0.8397`
- `graph_below_exact_queries = 0`

That establishes runtime safety at the latency target point. The remaining
validation question is whether the same ADR-031 runtime shape also preserves
live-vs-exact-quantized parity at the higher-`ef_search` settings used by the
recall gates.

## Problem

The real `50k` scratch fixture currently has the `m=8` index loaded, but not a
separate `m=16` index. Before deciding whether it is worth loading `m=16`, the
cheap next read is:

- keep `m=8`
- raise `ef_search` to `128` and `200`
- confirm that the live graph path still matches the exact quantized outputs

This does not replace the full recall gate, but it is the fastest way to detect
whether ADR-031 introduces any new higher-`ef_search` runtime distortion on the
already-loaded fixture.

## Planned Investigation

Use the external summary harness on the bounded real-`50k` query subset already
created in packet `283`:

- `tqhnsw_real_50k_queries_200_adr031`
- `tqhnsw_real_50k_m8_idx`
- `ef_search = 128`
- `ef_search = 200`

Record:

- `graph_recall_at_10`
- `exact_quantized_recall_at_10`
- `graph_below_exact_queries`
- `worst_exact_gap`

## Success Criteria

- the packet records the higher-`ef_search` summary commands used
- the packet records whether live graph results still match exact quantized
  outputs at `ef_search=128` and `200`
- the packet makes a clear call on whether ADR-031 validation is now sufficient
  on the loaded `m=8` fixture
