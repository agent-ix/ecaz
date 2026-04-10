# A4 Reference Curve vs Gate

## Context

Reviews 214 and 215 established two important points on the current uniform `10k` synthetic
workload:

- raw `hnsw-rs` code-graph at `m=8 / ef_search=128`: `30.5%`
- raw `hnsw-rs` source-graph at `m=8 / ef_search=128`: `29.0%`

That already ruled down tqvector persistence/runtime and quantization as the primary blocker at the
gate configuration.

## Goal

Check whether the strongest A4 reference configuration can even get close to the current gate on
the same workload.

## Probe

New ignored unit test:

- `tests::test_hnsw_rs_source_graph_recall_uniform_10k_m16_ef200`

Shape:

- corpus: uniform random unit vectors, `10,000 x 1536`
- queries: `20`
- graph builder/searcher: raw `hnsw-rs`, sequential insert
- objective: source-vector inner product
- settings: `m=16`, `ef_construction=128`, `ef_search=200`

## Result

Output:

- `hnsw-rs source graph timings: m=16 ef_search=200 build=741.168s search=6.594s`
- `hnsw-rs source graph probe: queries=20 m=16 ef_search=200 hnsw=0.6650`

So the strongest measured raw source reference point is:

- `Recall@10 = 66.5%`

## Comparison

Reference curve so far on the current uniform `10k` workload:

- source graph, `m=8 / ef_search=128`: `29.0%`
- source graph, `m=16 / ef_search=200`: `66.5%`

Current A4 gate:

- required: `Recall@10 >= 89%` at `m=8 / ef_search=128`

## Interpretation

This is the clearest contradiction yet.

On the current synthetic gate workload, even the raw source-vector reference HNSW does not come
close to the required recall:

- not at the gate point
- and not even at the strongest currently measured A4 configuration

That means the active blocker is no longer well-described as:

- a tqvector runtime bug
- a persistence bug
- or a quantizer-only bug

It is now a benchmark/fixture contradiction unless a stronger reference implementation can be shown
to achieve radically better recall on this exact workload.

## What This Does Not Mean

This does **not** mean "lower the standard".

It means the current gate setup and the current synthetic workload appear misaligned. If the gate
is real, then the fixture needs to represent a workload where HNSW can actually express that level
of neighborhood structure. If the fixture stays as-is, the reference curve says the requirement is
not attainable by the underlying algorithm at these settings.

## Next Step

The highest-value next move is no longer more tqvector runtime surgery. It is to resolve the
gate/fixture contradiction explicitly:

- confirm whether the intended A4 synthetic workload was really this uniform `10k` fixture
- compare against the spec/reviewer expectation for corpus shape
- and only then decide whether to keep debugging implementation, or to change the benchmark fixture
  to the intended workload before continuing A4
