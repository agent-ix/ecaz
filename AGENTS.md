# Agent Workflow

This repository uses a review-packet workflow in addition to normal code changes.

## Review Packet Rules

- Treat `review/README.md` as the source of truth for which review requests are open.
- At the start of a turn, read `review/README.md` first, then read only the currently open review request files named there.
- If new outside-review feedback is present for an open request, process that feedback before starting new implementation work.
- Do not close review requests yourself. You may add self-review notes or follow-up comments, but leave requests open until an outside reviewer has responded.
- Do not re-triage closed review files unless an outside reviewer reopens them.
- Keep external review bundles under `review/external/`.

## Checkpoint Rules

- Work in narrow, testable slices.
- After each code checkpoint, run:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Commit each green code checkpoint.
- After a checkpoint, add or update the matching review request in `review/` and commit that review-packet update separately.
- Push committed checkpoints and review-packet updates to `origin/main`.

## Local Safety Rules

- Do not revert unrelated local changes.
- Preserve the current on-disk layout unless a very small change is clearly justified.
- Add ADRs for design decisions that need durable rationale.
