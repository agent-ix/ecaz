# Review Request: C1 ADR-032 Full Layer-0 Source Promotion

## Context

Retrospective split from the original packet `293`.

After `297` showed that exact-on-head promotion was a real lever, this follow-up tried the most
direct low-`ef` quality-recovery idea: exact-score every layer-0 source before it is allowed to
expand.

## Attempt

- add a promotion-aware layer-0 search helper in `graph.rs`
- exact-score each candidate before it is allowed to expand as a layer-0 source
- requeue the candidate if exact scoring makes it worse than its current approximate rank

## Measurements

This attempt does not have a stable final latency summary row, and that absence is intentional in
this retrospective packet.

All known retained evidence for this attempt:

- initial implementation was invalid because it accidentally removed the original beam-search stop
  condition and degenerated into an effectively unbounded layer-0 walk
- after fixing that bug, the repaired variant still left the canonical warm real-`50k`,
  `m=8`, `ef_search=40` seam outside the old millisecond band, so it was discarded during
  diagnosis rather than run to a stable benchmark checkpoint
- `perf` on the repaired version while the warm `ef=40` cell was running showed:
  - `40.61%` `ProdQuantizer::score_ip_from_split_parts`
  - `7.98%` `graph::read_page_tuple`
  - `5.30%` `cached_graph_element`
  - `3.15%` `graph::pop_live_frontier_candidate`
  - `2.10%` `graph::push_frontier_and_result_candidate`

No trustworthy recall row was retained for this attempt because it was discarded before reaching a
stable benchmark/readout state.

## Outcome

Discarded.

Exact-promoting every popped layer-0 source was simply too expensive. The stop-condition bug was
fixed, but the scope of promotion was still far too wide to keep on the warm `ef=40` seam.
