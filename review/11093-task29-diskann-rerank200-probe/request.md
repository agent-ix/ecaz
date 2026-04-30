# Task 29 DiskANN Rerank 200 Probe

## Summary

This packet extends `11092` by testing whether a larger persisted SQL exact
rerank window can close the DiskANN SQL-vs-memory recall gap. The local
baseline index was temporarily altered to `rerank_budget=200`, measured through
`ecaz-cli`, then restored to its original reloptions.

Commands used explicit local PG18 targeting and packet-local `--log-output`
artifacts.

## Result

Raising `rerank_budget` from `100` to `200` improves aggregate recall again,
but it still does not close the SQL-vs-memory gap:

- rerank_budget `200`, valid sweep:
  - `list_size=200`: recall `0.9845`, NDCG `0.9990`, mean `143.63 ms`
  - `list_size=400`: recall `0.9845`, NDCG `0.9990`, mean `187.27 ms`
  - `list_size=800`: recall `0.9845`, NDCG `0.9990`, mean `327.79 ms`

The five-query comparison at `scan_list_size=200` still shows query `10001`
missing the same two exact IDs:

- in-memory source-vector graph recall for the full 200-query probe: `1.0000`
- query `10001` exact/memory: `10/10`
- query `10001` exact/sql: `8/10`
- SQL still substitutes `9976,9999` for exact IDs `9717,7782`

The index was restored afterward to:

```text
{graph_degree=32,build_list_size=100,alpha=1.2}
```

## Recommendation

`rerank_budget=200` is a strong enough tuning knob to make local real-10k recall
look much healthier, moving from roughly `0.931` at the default to `0.9845`.
It is not sufficient as a landing answer because a known query miss survives
both larger rerank windows and larger `list_size` values.

The next optimization should target persisted traversal scoring before exact
rerank. Add a focused persisted-frontier diagnostic for query `10001` that
reports whether exact IDs `9717` and `7782` ever enter the grouped-PQ frontier,
their pre-rerank ranks if present, and the competing IDs that displace them.

## Artifacts

See `artifacts/manifest.md`.
