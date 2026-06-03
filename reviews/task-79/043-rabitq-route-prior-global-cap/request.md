# Task 79 Review Request: RaBitQ Route Prior Global Cap

## Summary

This packet reviews source checkpoint `19845a140`, which adds a default-off SPIRE route-score prior for global leaf block pruning.

The change is narrow:

- adds `ec_spire.leaf_block_pruning_route_prior_weight`, default `0.0`, range `0.0..1.0`;
- carries routing score into recursive leaf routes and leaf object read routes;
- folds `route_prior_weight * route_score` into global leaf block scoring only when the GUC is nonzero;
- keeps direct/snapshot leaf paths at route score `0.0`;
- leaves the default behavior unchanged.

This was meant to test whether a routing-quality prior can make lower global block caps recall-preserving, directly addressing the Task 79 candidate-surface problem.

## Validation

Packet-local focused tests passed:

- `artifacts/cargo-test-route-prior.log`: `score_global_leaf_block_row_ranges_can_apply_route_prior`
- `artifacts/cargo-test-zero-route-prior.log`: `select_global_leaf_block_row_ranges_spends_budget_across_leaves`

Local PG18 install/restart completed:

- backend SHA256: `239f288f79d512ef43dbcadfe3181861d9d2465cc2c2a0ea5f9ad3c6e6ba2774`
- `artifacts/install-route-prior-ecaz-pg18.log`
- `artifacts/pg18-restart-route-prior.log`

`ecaz bench suite` audit/status completed with 13/13 successful steps:

- `artifacts/suite-audit.log`
- `artifacts/suite-status.log`

All work in this packet is local PG18 only. No AWS was used.

## Benchmark Result

Suite config: `suite-rabitq-route-prior-global-cap.json`

Measured shape: RaBitQ, block16, k=3 summaries, nprobe 96, per-leaf block cap disabled, global block cap varied, radius weight 0.25, rerank width 25, 200 queries.

| row | global blocks | route prior weight | candidates | candidate delta vs 1216 | latency p50 ms | latency p95 ms | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| global1216_rp000 | 1216 | 0.00 | 3,877,368 | 0 | 56.145 | 65.566 | 0.9940 |
| global1024_rp000 | 1024 | 0.00 | 3,265,373 | -611,995 | 51.254 | 58.828 | 0.9920 |
| global896_rp000 | 896 | 0.00 | 2,857,174 | -1,020,194 | 51.176 | 61.618 | 0.9900 |
| global768_rp000 | 768 | 0.00 | 2,449,116 | -1,428,252 | 49.375 | 60.366 | 0.9865 |
| global1024_rp005 | 1024 | 0.05 | 3,264,990 | -612,378 | 51.219 | 58.757 | 0.9920 |
| global896_rp005 | 896 | 0.05 | 2,857,058 | -1,020,310 | 50.413 | 58.340 | 0.9900 |
| global768_rp005 | 768 | 0.05 | 2,449,011 | -1,428,357 | 48.318 | 53.177 | 0.9865 |

The full matrix also includes route prior weights `0.02` and `0.10`; those rows are in `artifacts/compact-results.tsv`.

## Interpretation

The direct global cap reduction works mechanically: it removes 0.61M to 1.43M candidates locally and improves p50 latency from 56.145ms to as low as 48.318ms.

The route-prior idea does not solve the recall problem. At 1024, 896, and 768 global blocks, route prior weights `0.02`, `0.05`, and `0.10` all preserve the same recall as the zero-prior row for that cap. They do not recover the 1216-block recall of 0.9940.

So the source hook is useful as a default-off diagnostic, but this packet is a negative result for route-prior as the candidate-surface strategy. The next Task 79 slice should move away from simple route-prior scoring and toward a more direct recall-preserving candidate reduction mechanism, such as better global block selection features, per-query adaptive caps, or a second-stage verifier that gates object reads before the expensive candidate surface expands.

## Review Focus

Please review:

- whether the default-off route-prior hook is acceptable to keep as diagnostic plumbing;
- whether the benchmark conclusion is sound;
- whether the next slice should abandon route prior and target a different direct candidate-surface reducer.

Artifacts are summarized in `artifacts/manifest.md`.
