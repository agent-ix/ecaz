# Review Request: C1 Greedy Upper-Layer Seeding

## Context

Packet `255` showed that fast hash state is worth keeping but is not the next
big latency unlock. After the score-cache and fast-hash slices, the canonical
real `10k` `m=8` surface still sits around:

- `ef_search=40`: mean `88.360ms`
- `ef_search=200`: mean `174.147ms`

The post-fast-hash representative hot-path probe still shows large rescan
seeding cost:

- `ef_search=40`
  - upper-layer seed elapsed: `24.271ms`
  - layer-0 seed elapsed: `14.207ms`
- `ef_search=200`
  - upper-layer seed elapsed: `61.339ms`
  - layer-0 seed elapsed: `66.907ms`

That makes upper-layer seed search one of the remaining highest-signal C1
targets.

## Problem

The scan runtime currently uses a full result-window search across every upper
layer during `amrescan` seeding. That is more expensive than the classic HNSW
pattern, which greedily descends upper layers to a single best local optimum
and only opens the wider beam at layer 0.

The rest of the codebase already trusts greedy upper-layer descent for:

- insert search
- vacuum repair search

So the scan runtime is now the outlier.

## Planned work

1. Switch scan-time upper-layer seeding from per-layer result-window search to
   cached greedy descent.
2. Keep layer-0 search behavior unchanged.
3. Re-run the existing validation suite, including the recall gate already
   exercised by `cargo pgrx test pg17`.
4. Re-run the representative hot-path probe and canonical `m=8` verified
   surface to see whether the upper-layer bucket collapses materially without
   compromising behavior.

## Exit criteria

- a pushed checkpoint materially reduces upper-layer seed time on the real C1
  path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records measured before/after evidence, including whether the
  greedy shift was worth keeping
