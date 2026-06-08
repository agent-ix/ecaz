# Task 84 Review Request: Multi-Index KNN Frontdoor Fix and k=3 AWS Result

## Summary

Packet 004's AWS query-only reruns failed before recall measurement with:

`ERROR: ec_spire_distributed: relation context could not be loaded`

The root cause was the retained AWS 1M comparison setup: the corpus table has
both the retained k=2 SPIRE index and the new k=3 SPIRE index. The ADR-069 DML
front-door relation-context loader rejects multiple SPIRE indexes, and the
planner hook was fail-closing even for non-PK vector KNN reads.

This packet lands the planner-hook fix and reruns the retained k=3 index
query-only suite.

## Code Change

Commit: `07974586f` (`Allow multi-index SPIRE KNN reads through DML frontdoor`)

- Keeps the existing multi-index fail-closed behavior for ADR-069 PK/DML
  front-door candidates.
- Allows unrelated non-PK SELECTs, including vector KNN benchmark reads, to
  pass through when relation-context loading fails due a multi-index table.
- Adds a PG18 regression for multi-index SPIRE KNN pass-through.
- Re-runs the existing context-error fail-closed PG18 test.

## Validation

- `cargo pgrx test pg18 test_ec_spire_multi_index_knn_select_passes_through`
  passed.
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_hook_fail_closed_context_error`
  passed.
- `cargo build -p ecaz-cli` passed with the existing
  `LoadedDistributedPlacementConfig.path` warning.

## AWS Run

The successful install used:

`target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-84-spire-recall-recovery --skip-extension-recreate`

The successful benchmark used:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-query-only-q500.json`

The suite completed all three q500 pipeline rows plus storage. The direct
retained-budget comparison is `global1152`.

## Results

| cap | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| global1024 | 0.9808 | 8,190,090 | 12,500 | 264.202 ms | 340.013 ms | 2902.047 ms |
| global1152 | 0.9832 | 9,213,742 | 12,500 | 252.186 ms | 311.879 ms | 324.200 ms |
| global1280 | 0.9846 | 10,237,430 | 12,500 | 258.684 ms | 314.655 ms | 330.054 ms |

The `global1152` k=3 row has the same recall as the retained k=2 baseline:

- retained baseline: `recall@10=0.9832`, `candidate_sum=9,213,846`
- k=3 row: `recall@10=0.9832`, `candidate_sum=9,213,742`

The miss-attribution split at `global1152` is also unchanged:

- `3` routing misses
- `81` selected-leaf block-pruning/candidate-cap misses

At `global1280`, k=3 matches the Task 83 blanket-cap control neighborhood:

- `recall@10=0.9846`
- `candidate_sum=10,237,430`
- miss split: `3` routing misses, `74` selected-leaf misses

Storage:

- k=3 index: `936.4 MiB`, `991.9 B/row`
- retained k=2 index: `872.1 MiB`, `923.7 B/row`

## Outcome

The multi-index KNN front-door fix is a valid prerequisite and should remain.
It enables side-by-side SPIRE index comparisons on a retained benchmark table
without weakening ADR-069 fail-closed behavior.

The k=3 summary-representative policy does **not** recover Task 84 recall on
AWS 1M/q500. It preserves the candidate surface but lands exactly on the
retained `0.9832` recall point, and broader caps behave like the existing
blanket-cap controls.

## Requested Review

Please review:

- the narrowness of the multi-index KNN pass-through fix;
- the PG18 validation coverage for pass-through plus fail-closed behavior;
- the AWS k=3 interpretation as a negative result for the k>2 summary-count
  axis.

Recommended next Task 84 direction: close the k>2 summary-count axis and move
to a bounded selective-rescue policy keyed by selected-leaf ambiguity/near-cap
signals, since route-prior and richer summary count have not beaten the Task
79/81 retained point.
