# Task 29 DiskANN Grouped Frontier Probe

## Summary

This packet uses the new `ecaz-cli` grouped-PQ frontier diagnostic from
`eb826920` to inspect the known miss from packets `11091` through `11093`.
The command rebuilds the source-vector Vamana graph for the local real-10k
corpus, retrains the same grouped-PQ4 search-code model shape used by
`ec_diskann`, and simulates the persisted grouped-PQ traversal frontier for
query `10001`.

The probe used `scan_list_size=200` and a simulated `rerank_budget=200`.

## Result

The two exact IDs that SQL keeps missing for query `10001` do not reach the
grouped-PQ frontier at all:

- in-memory source-vector graph recall@10 for the 200-query probe: `1.0000`
- grouped-PQ simulated reranked IDs:
  `8885,9785,9957,9826,9926,9944,9855,9915,9976,9999`
- exact ID `9717`: exact rank `5`, grouped-PQ frontier rank `missing`
- exact ID `7782`: exact rank `10`, grouped-PQ frontier rank `missing`

Other exact IDs are present and inside the simulated rerank budget:

- `9785`: frontier rank `31`
- `9926`: frontier rank `48`
- `9944`: frontier rank `28`
- `9855`: frontier rank `12`
- `9915`: frontier rank `10`

This matches the SQL symptom from `11093`: rerank can reorder candidates it
sees, but it cannot recover `9717` or `7782` because grouped-PQ traversal does
not keep them in the candidate frontier.

## Recommendation

The first landing blocker is not Vamana graph construction and not merely the
exact rerank budget. It is grouped-PQ traversal recall.

Next optimization should change the pre-rerank candidate generation path, not
the final exact heap rerank. The lowest-risk next probe is to add a dual
frontier mode to the CLI diagnostic that compares current grouped-PQ traversal
against exact source-vector traversal for the same graph and reports the first
hop/rank where `9717` and `7782` diverge. That will tell us whether to tune the
grouped-PQ model shape, increase traversal breadth beyond the exposed
`list_size`, or add a hybrid exact/top-up pass before rerank.

## Artifacts

See `artifacts/manifest.md`.
