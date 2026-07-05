# A4 Synthetic vs NFR Dataset Gap

## Context

The latest reference baselines showed that raw source-vector HNSW is already weak on the current
uniform synthetic fixture:

- uniform `10k`, source graph, `m=8 / ef_search=128`: `29.0%`
- uniform `10k`, source graph, `m=16 / ef_search=200`: `66.5%`

That raised the next question:

- can the existing in-repo clustered synthetic generator serve as the "equivalent dataset" allowed
  by `NFR-003`, or is A4 now blocked on a real external embedding corpus?

## Spec Tension

There is an explicit requirement split now visible in the repo:

- `plan/tasks/05-graph-scan.md` A4 says: measure Recall@10 on synthetic data
- `spec/non-functional/NFR-003-recall-quality.md` says: recall benchmarks SHALL run against the
  DBpedia OpenAI embeddings dataset, or equivalent

So if the synthetic fixture is going to carry the A4 gate, it has to behave like a credible
equivalent dataset, not just be cheap to generate.

## Probe

New ignored unit test:

- `tests::test_hnsw_rs_source_graph_recall_clustered_10k`

Shape:

- corpus: clustered Gaussian-mixture synthetic, `10,000 x 1536`, `50` clusters, spread `0.3`
- queries: `20`
- graph builder/searcher: raw `hnsw-rs`, sequential insert
- objective: source-vector inner product
- settings: `m=8`, `ef_construction=128`, `ef_search=128`

## Result

Output:

- `hnsw-rs source graph timings: m=8 ef_search=128 build=401.365s search=5.455s`
- `hnsw-rs source graph clustered probe: queries=20 m=8 ef_search=128 hnsw=0.2600`

So the authoritative clustered synthetic reference point is:

- `Recall@10 = 26.0%`

## Interpretation

This rules down the obvious escape hatch.

The existing clustered synthetic generator is **not** a convincing equivalent to the real embedding
benchmark surface required by `NFR-003`. It behaves almost the same as the uniform generator at the
gate configuration:

- uniform source HNSW, `m=8 / ef_search=128`: `29.0%`
- clustered source HNSW, `m=8 / ef_search=128`: `26.0%`

So changing from "uniform synthetic" to "current clustered synthetic" does not resolve the A4
contradiction.

## Current Read

At this point:

1. tqvector runtime bugs were real, but they are no longer the main blocker
2. raw reference HNSW is weak on the current synthetic fixtures
3. the current in-repo clustered synthetic helper is not a sufficient substitute for the NFR
   dataset requirement

That means the next serious benchmark lane is no longer "pick a better random generator". It is:

- run A4 against a real embedding corpus, or
- add an external-corpus loader path that lets the existing durable recall probes run against one

## Next Step

Implement or reuse a path that measures the existing relation-based recall probes on an external
embedding table, then run A4 on a real `1536`-dimensional corpus consistent with `NFR-003`.
