# Review Request: Target Candidate Rank Diagnostics

- task: Task 120
- packet: `reviews/task-120/003-target-candidate-rank`
- code commit under review: `ad99a4db8e3f7a5c8fcf8d50eed9065daada3bd8`
- parent packet: `reviews/task-120/002-stage-containment-jsonl`

## Summary

This slice replaces the candidate/rerank lower-bound portion of
`spire_stage_containment` with an exact target candidate-rank diagnostic.

Code changes:

- Adds `ec_spire_index_scan_target_candidate_rank_snapshot(index_oid, query, target_local_sequences)` to expose each requested truth row's approximate candidate rank, candidate frontier size, rerank prefix size, rerank-prefix membership, placement, row identity, approximate score, and heap TID.
- Reuses the same approximate candidate collection path as normal SPIRE local scan diagnostics; it does not change candidate selection, rerank behavior, index layout, or defaults.
- Updates `ecaz bench spire-pipeline --stage-containment-output` to query the new snapshot and classify:
  - `local_candidate_frontier` from `target_candidate_rank_snapshot`
  - `exact_source_rerank_frontier` from `target_candidate_rank_snapshot`
  - missing reasons including `candidate_not_retained` and `rerank_width_cap`
- Keeps block/routing attribution on the existing target block-rank snapshot.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md` for provenance.

- `cargo fmt --check` passed.
- `cargo test -p ecaz-cli spire_pipeline` passed: 22 tests, including `stage_containment_records_per_stage_truth_retention`.
- `cargo test -p ecaz target_candidate_rank` passed as an extension test-harness compile/filter check.

## Review Focus

- Confirm the new scan helper observes the same approximate frontier as the existing local scan diagnostic path.
- Confirm the SQL-facing row schema is sufficient for Phase 1 containment evidence without changing scan/rerank behavior.
- Confirm the CLI stage containment reasons now distinguish routing miss, block prune, candidate non-retention, rerank-width cap, and final top-k miss.

## Closeout Status

This is still a measurement-surface checkpoint, not Task 120 closeout. It does
not include staged 10k/50k/100k `ecaz bench suite` evidence, and it does not
claim the task is complete.
