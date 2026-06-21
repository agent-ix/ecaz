# Review Request: Target Candidate Rank JSONL Output

- task: Task 120
- packet: `reviews/task-120/005-target-candidate-rank-output`
- code commit under review: `9178f4bc04743018b2302859780a5e21b763b3ce`

## Summary

This slice makes the target candidate-rank diagnostic durable in benchmark
packets.

Code changes:

- Adds `ecaz bench spire-pipeline --target-candidate-rank-output <path>`.
- Writes one `spire_target_candidate_rank` JSONL row per exact truth neighbor,
  including approximate candidate rank, rerank-prefix membership, candidate
  frontier size, rerank prefix size, placement, row identity, approximate score,
  and heap TID.
- Reuses the target candidate-rank query when both
  `--target-candidate-rank-output` and `--stage-containment-output` are enabled
  for the same query.
- Adds `target_candidate_rank_output` to `ecaz bench suite` expansion and
  expected-artifact tracking.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.

- `cargo fmt --check` passed.
- `cargo test -p ecaz-cli spire_pipeline` passed: 22 tests.
- `cargo run -p ecaz-cli -- bench spire-pipeline --help` passed and shows the
  new `--target-candidate-rank-output` flag.

## Closeout Status

This is still diagnostic plumbing. Task 120 remains open pending actual
10k/50k/100k `ecaz bench suite` measurement evidence.
