# Frontier Smoke 10k TurboQuant ef=200 Summary

Derived from the uncommitted smoke JSONL with:

```bash
jq -r '[.ef_search, .query_index, .pre_final_frontier_size, (.frontier_row_indices|length), .final_emitted_count, (.final_emitted_row_indices|length), .truth_top10_in_frontier] | @tsv' \
  reviews/task-118/014-candidate-pool-diagnostic-correction/artifacts/frontier-smoke-10k-turboquant-ef200.jsonl
```

| ef_search | query_index | pre_final_frontier_size | frontier_row_count | final_emitted_count | final_emitted_row_count | truth_top10_in_frontier |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 200 | 0 | 200 | 200 | 200 | 200 | 10 |
| 200 | 1 | 200 | 200 | 200 | 200 | 10 |
| 200 | 2 | 200 | 200 | 200 | 200 | 10 |
| 200 | 3 | 200 | 200 | 200 | 200 | 10 |
| 200 | 4 | 200 | 200 | 200 | 200 | 10 |
