# Review Request: C1 Scan Score Cache

## Context

Packet `253` established that the remaining ordered-scan cost is dominated by
candidate scoring, not tuple reads.

Representative real `10k` hot-path profile for query `id=10000`, `m=8`:

- `ef_search=40`
  - `candidate_score_calls = 1125`
  - `candidate_score_elapsed_us = 76477`
  - graph tuple load time stayed around `4ms`
- `ef_search=200`
  - `candidate_score_calls = 4915`
  - `candidate_score_elapsed_us = 360059`
  - graph tuple load time stayed around `15ms`

The same profile also showed substantial element reuse:

- `graph_element_cache_hits = 804` at `ef_search=40`
- `graph_element_cache_hits = 3926` at `ef_search=200`

So repeated visits during one ordered scan are still paying the same
query/code score cost over and over.

## Problem

The current ordered scan has a scan-local graph tuple cache, but not a
scan-local score cache. Repeated visits to the same element TID still rerun
`score_scan_element_result(...)` against the same query and code bytes.

That is now the highest-signal remaining C1 target.

## Planned work

1. Add a scan-local score cache keyed by element TID for the ordered scan.
2. Route the graph search path through that cache so repeated element visits do
   not rescore identical query/code pairs within one scan.
3. Re-run the hot-path profile from packet `253` plus representative SQL probes
   to confirm:
   - score-call count drops materially
   - scoring time drops materially
   - wall-clock latency moves in the right direction
4. Keep the slice narrow:
   - no planner changes
   - no harness changes
   - no speculative graph algorithm rewrite

## Exit criteria

- a pushed checkpoint materially reduces repeated scoring work on the real C1
  ordered scan path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- the packet records before/after hot-path evidence, not just code intent
