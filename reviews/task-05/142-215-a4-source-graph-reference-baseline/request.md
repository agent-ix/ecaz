# A4 Source-Graph Reference Baseline

## Context

Review 214 established that the pure-Rust `hnsw-rs` code-graph baseline is already weak on the
same uniform `10k` synthetic distribution used by the SQL fixture:

- `hnsw-rs` code-graph Recall@10: `30.5%`
- build-code brute force overlap: `80.5%`
- exact quantized overlap: `84.0%`

That still left one important possibility:

- maybe the low reference result is mostly caused by the code-code graph objective, rather than
  the HNSW operating point itself

## Goal

Compare the raw `hnsw-rs` baseline built on source vectors against the raw `hnsw-rs` baseline
built on code vectors, using the same `10k x 1536`, `m=8`, `ef_construction=128`,
`ef_search=128` operating point.

## Experiments

### 1. Exploratory parallel-insert source graph

I first used `parallel_insert_slice` for a fast directional answer.

Results:

- uniform `10k`: `24.0%`
- clustered `10k`: `26.5%`

These were useful only as hints. They are not authoritative, because the real tqvector build path
uses sequential insertion, not `hnsw-rs` parallel insert.

## 2. Authoritative sequential source graph

I then reran the source-graph baseline with sequential `hnsw.insert(...)` to match the real build
regime.

Output:

- `hnsw-rs source graph timings: m=8 ef_search=128 build=402.999s search=5.569s`
- `hnsw-rs source graph probe: queries=20 m=8 ef_search=128 hnsw=0.2900`

So the authoritative source-vector baseline is:

- raw source-graph HNSW Recall@10: `29.0%`

## Comparison

At the same operating point on the same uniform `10k` workload:

- source-graph HNSW: `29.0%`
- code-graph HNSW: `30.5%`
- build-code brute force: `80.5%`
- exact quantized brute force: `84.0%`

## Interpretation

This rules down another major branch.

The low A4 behavior is not primarily caused by:

- tqvector persistence/runtime
- the 4-bit quantized graph objective by itself

Because even raw source-vector HNSW at the same `m=8 / ef_construction=128 / ef_search=128`
operating point is still only `29.0%`.

The dominant issue is now the HNSW operating point relative to the current synthetic workload.

## What This Means

The active contradiction is no longer "why is tqvector much worse than the reference graph?".
The reference graph itself is bad at the gate operating point.

That shifts the center of gravity toward:

- synthetic workload / gate mismatch
- HNSW parameter ceiling on this workload
- or a deeper library-level/reference-level construction limitation

It shifts away from:

- more tqvector runtime surgery
- more lower-page serialization debugging
- more quantizer-only blame for the remaining graph miss

## Next Step

The next efficient follow-ups are:

- widen the reference sweep to the other A4 configurations, especially `m=16, ef_search=200`
- compare that reference curve against the SQL gate curve
- then decide whether A4 is blocked by an implementation bug or by a benchmark/fixture mismatch
