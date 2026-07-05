# Task 29 DiskANN Rerank Budget Probe

## Summary

This packet tests whether the SQL-vs-memory recall gap from `11091` is caused
primarily by the persisted SQL scan's `rerank_budget=64` default. The local
baseline index was temporarily altered to `rerank_budget=100`, measured through
`ecaz-cli`, then restored to its original reloptions.

Commands used explicit local PG18 targeting and packet-local `--log-output`
artifacts.

## Result

Raising `rerank_budget` improves aggregate recall but does not close the gap:

- baseline prior packet: recall was about `0.931` across `list_size=64..800`
- rerank_budget `100`, valid sweep:
  - `list_size=100`: recall `0.9550`, NDCG `0.9976`, mean `81.96 ms`
  - `list_size=128`: recall `0.9550`, NDCG `0.9976`, mean `85.36 ms`
  - `list_size=200`: recall `0.9555`, NDCG `0.9976`, mean `99.95 ms`
  - `list_size=400`: recall `0.9555`, NDCG `0.9977`, mean `139.34 ms`
  - `list_size=800`: recall `0.9555`, NDCG `0.9977`, mean `281.22 ms`

The five-query comparison at `list_size=100` still shows query `10001` missing
the same two exact IDs:

- exact/memory: `10/10`
- exact/sql: `8/10`
- SQL still substitutes `9976,9999` for exact IDs `9717,7782`

The index was restored afterward to:

```text
{graph_degree=32,build_list_size=100,alpha=1.2}
```

## Recommendation

`rerank_budget` is a useful tuning knob, but not the landing fix. It moves
recall from roughly `0.931` to `0.9555`, while in-memory source-vector graph
search remains `0.9995`.

Next target: persisted traversal scoring. The misses that survive a larger
rerank window are likely caused by the grouped-PQ traversal score failing to
keep exact neighbors in the frontier before exact rerank can see them. The next
probe should compare, for query `10001`, the persisted traversal frontier
before rerank against exact source-vector graph traversal and report whether
IDs `9717` and `7782` are in the persisted frontier at all.

## Artifacts

See `artifacts/manifest.md`.
