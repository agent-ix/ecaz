# Agent Workflow

This repository uses a review-packet workflow in addition to normal code changes.

## Review Packet Rules

See `review/README.md` for full structure and conventions.

### Structure

Each review topic is a directory under `review/`:

    review/{NN}-{topic}/
      request.md
      feedback/
        {YYYY-MM-DD}-{seq}-{agent}.md

### Agents and Number Ranges

| Name     | Role     | Range       | Area                |
|----------|----------|-------------|---------------------|
| coder1   | coder    | 1–9999      | core scan/build/index |
| coder2   | coder    | 10000–19999 | planner integration |
| reviewer | reviewer | —           | reviews any topic   |

### Feedback Files

- Filename: `{YYYY-MM-DD}-{seq}-{agent}.md`
- Every feedback file must include frontmatter with `agent`, `role`, `model`, `date`, and `seq` fields.
- Any agent can leave feedback on any topic.

### Workflow

- At the start of a turn, scan `review/` for topics with new feedback files you haven't processed.
- If new feedback is present for a topic you own, process it before starting new implementation work.
- Do not close review requests yourself. Leave requests open until an outside reviewer has responded.
- Do not re-triage closed review topics unless an outside reviewer reopens them.
- Treat each significant implementation or investigation attempt as its own review topic.
- A review packet draft is only for preserving in-progress work on the current attempt if that
  attempt runs long.
- If the approach, hypothesis, or measurement seam changes in a meaningful way, start a new review
  packet before continuing.
- Do not append multiple distinct attempts to one long-lived packet just because they are related
  to the same ADR or milestone.
- Failed attempts still need their own packet so the branch preserves a readable experiment log.

## Checkpoint Rules

- Work in narrow, testable slices.
- Keep packet scope narrow: one packet should correspond to one concrete slice or experiment.
- After each code checkpoint, run:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Commit each green code checkpoint.
- If a slice is still in progress after a while, update that slice's packet draft before continuing
  so partial work is recorded.
- After a checkpoint, add or update the matching review request in `review/` and commit that
  review-packet update separately.
- When starting a new significant attempt, create a new packet instead of reusing the previous
  attempt's packet.
- Record all known measurements for that attempt in the packet, not just one summary number.
- If an attempt has multiple runs, list each run's result or explicitly account for why a run was
  discarded as invalid/noise.
- If one result is called out as representative, label it as representative without omitting the
  other known results for that attempt.
- Push committed checkpoints and review-packet updates to the remote immediately after committing. Feedback that exists only locally is invisible to other agents.

## Reviewer Workflow

- When leaving feedback on a branch, commit to **that branch** and push immediately.
- If reviewing multiple branches, commit and push feedback to each branch separately.
- Never run destructive git operations (reset, rebase, drop commits) without reading the affected commits and getting explicit confirmation from the user first.
- After pushing, verify the push succeeded before moving on.

## Local Safety Rules

- Do not revert unrelated local changes.
- Preserve the current on-disk layout unless a very small change is clearly justified.
- Add ADRs for design decisions that need durable rationale.
