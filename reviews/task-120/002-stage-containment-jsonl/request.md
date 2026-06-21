# Review Request: Stage Containment JSONL

## Summary

This checkpoint adds a Task 120 Phase 1 output surface to
`ecaz bench spire-pipeline`:

- new CLI flag: `--stage-containment-output <path>`
- new suite JSON field for `spire-pipeline` steps:
  `stage_containment_output`

The output writes one `spire_stage_containment` JSONL row per query and stage:

- `topology_route_set`
- `selected_leaves`
- `selected_leaf_blocks`
- `local_candidate_frontier`
- `exact_source_rerank_frontier`
- `final_top_k`

Each row includes exact-truth top-k containment counts/ranks, missing reason
counts, local pipeline budget/blocker fields, leaf/block byte counters, and
available stage latency counters.

Important precision note: selected-leaf and selected-block containment are
derived from the existing target block-rank snapshot. Candidate and rerank
frontier containment is currently reported as a final-hit lower bound and is
marked with
`final_hits_lower_bound_until_target_candidate_rank_snapshot`. A later Phase 1
slice still needs a target candidate-rank SQL snapshot to split candidate-budget
drops from rerank/final-top-k drops precisely.

No scan behavior, index format, or default policy changes in this slice.

## Validation

- `cargo fmt --package ecaz-cli --check` passed with existing stable-rustfmt
  warnings.
- `cargo test -p ecaz-cli spire_pipeline` passed: `22 passed; 0 failed`.

## Follow-Up

Use this output in the first `ecaz bench suite` Phase 1 matrix, alongside the
existing funnel and miss-attribution outputs. Do not treat this packet as Task
120 closeout evidence; it is diagnostic plumbing for the required containment
runs.
