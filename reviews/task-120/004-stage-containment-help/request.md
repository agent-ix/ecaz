# Review Request: Stage Containment Help Text

- task: Task 120
- packet: `reviews/task-120/004-stage-containment-help`
- code commit under review: `2fd07bef5330481b90b366a428553d7f8a807e4f`

## Summary

Packet `003-target-candidate-rank` replaced the lower-bound candidate/rerank
containment basis with the target candidate-rank SQL snapshot. This follow-up
updates the `--stage-containment-output` help text so the CLI documentation no
longer says candidate/rerank containment is a lower bound.

No runtime behavior changed.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.

- `cargo fmt --check` passed.
- `target/debug/ecaz bench spire-pipeline --help` passed and shows:
  `Candidate/rerank containment uses the target candidate-rank SQL snapshot.`

## Closeout Status

This is documentation cleanup only. Task 120 remains open pending real
10k/50k/100k `ecaz bench suite` evidence and later phase decisions.
