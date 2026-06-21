# Task 120 / 004 Stage Containment Help Artifacts

- head SHA: `2fd07bef5330481b90b366a428553d7f8a807e4f`
- task bucket: `reviews/task-120/004-stage-containment-help`
- lane: local validation
- fixture: not applicable; CLI help text only
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable
- timestamp: `2026-06-21T15:02:04Z`

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- key line: `Script done on 2026-06-21 08:01:45-07:00 [COMMAND_EXIT_CODE="0"]`
- note: stable rustfmt emitted the repo's existing unstable-option warnings.

### `ecaz-spire-pipeline-help.log`

- command: `target/debug/ecaz bench spire-pipeline --help`
- result: passed
- key lines:
  - `Requires --include-recall and query metrics. Candidate/rerank containment uses the target candidate-rank SQL snapshot.`
  - `Script done on 2026-06-21 08:01:53-07:00 [COMMAND_EXIT_CODE="0"]`
