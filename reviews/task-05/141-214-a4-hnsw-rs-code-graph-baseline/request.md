# A4 hnsw-rs Code-Graph Baseline

## Context

Review 213 left the lane at an uncomfortable split:

- live persisted graph on the fixed uniform `10k` fixture was still only around `37%`
- exact quantized on the same sample was `83.4%`
- build-code brute force on the same sample was `80.2%`
- corrected layer-1 neighborhood coverage was also low

That still left open a major question:

- is the remaining loss mostly in tqvector persistence/runtime?
- or does the same loss already exist in the reference `hnsw-rs` graph search before persistence?

## Goal

Run a pure-Rust `hnsw-rs` search probe on the same uniform `10k` synthetic distribution used by
the SQL recall fixture, using the same `m=8`, `ef_construction=128`, and 4-bit tiled-1536 code
objective that tqvector uses for graph build.

## Probe

New ignored unit test:

- `tests::test_hnsw_rs_code_graph_recall_uniform_10k`

Shape:

- corpus: uniform random unit vectors, `10,000 x 1536`
- queries: `20`
- quantizer: current production `1536 @ 4-bit` tiled-FWHT path
- graph objective: code-code inner product, same objective tqvector uses during graph build
- graph builder/searcher: raw `hnsw-rs`
- settings: `m=8`, `ef_construction=128`, `ef_search=128`

## Result

Output:

- `hnsw-rs code graph timings: m=8 ef_search=128 build=281.090s search=59.147s`
- `hnsw-rs code graph probe: queries=20 m=8 ef_search=128 hnsw=0.3050 build_code=0.8050 exact=0.8400`

So on the same operating point:

- `hnsw-rs` graph search Recall@10 against fp32 truth: `30.5%`
- build-code brute force overlap against fp32 truth: `80.5%`
- exact quantized overlap against fp32 truth: `84.0%`

## Interpretation

This is the strongest A4 localization result so far.

The dominant miss already exists in the reference in-memory graph search. It is not primarily a
Postgres persistence bug and not primarily a tqvector scan-runtime bug.

What this rules down:

- serialized neighbor corruption as the primary explanation
- lower-page traversal/runtime behavior as the primary explanation
- "if persistence were fixed, recall would jump near exact" as the default assumption

What it points to instead:

- the graph construction/search operating point itself is weak on this uniform `10k` corpus
- specifically, `m=8 / ef_construction=128 / ef_search=128` over the current 4-bit code objective
  produces poor HNSW recall long before tqvector persistence enters the picture

## Important Nuance

This probe used the build-time code-code objective, not the live asymmetric query scorer.

That nuance does not change the main conclusion, because the build-code brute-force ceiling on the
same corpus is still `80.5%`. The large gap is between brute-force on that objective and HNSW
search on that objective.

## Current Read

The center of gravity has shifted again:

1. hierarchy collapse was real and fixed
2. persistence/runtime bugs were real, but not dominant enough to explain A4
3. the remaining big miss is already present in raw `hnsw-rs`
4. the active problem is now graph quality / operating point, not just tqvector traversal logic

## Next Step

Two follow-ups matter most now:

- run the same pure-Rust baseline on a source-vector graph to see whether the weak result is
  specific to the code-code build objective
- widen the pure-Rust search sweep (`ef_search`, then possibly `m=16`) before doing more invasive
  tqvector runtime surgery
