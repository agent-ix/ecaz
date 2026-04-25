# Agent Workflow

This repository uses a review-packet workflow in addition to normal code changes.

## Review Packet Rules

See `review/README.md` for full structure and conventions.

### Structure

Each review topic is a directory under `review/`:

    review/{NN}-{topic}/
      request.md
      artifacts/
        manifest.md
        ...
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

### Measurement Artifacts

- Any review packet that makes a measurement claim should store the cited raw logs inside that packet's `artifacts/` directory instead of relying only on `tmp/`.
- Measurement packets should include `artifacts/manifest.md` as the packet-local source of truth for artifact metadata.
- `manifest.md` should record, for each artifact:
  - head SHA
  - packet/topic
  - lane / fixture / storage format / rerank mode
  - command used
  - timestamp
  - whether the run used isolated one-index-per-table or shared-table surfaces
  - the key result lines that `request.md` cites
- `request.md` should summarize the result and point at the packet-local artifact files.

### Workflow

- At the start of a turn, scan `review/` for topics with new feedback files you haven't processed.
- If new feedback is present for a topic you own, process it before starting new implementation work.
- Do not close review requests yourself. Leave requests open until an outside reviewer has responded.
- Do not re-triage closed review topics unless an outside reviewer reopens them.

## Checkpoint Rules

- Work in narrow, testable slices.
- Do not run tests by default. Run tests only when a change is risky enough
  that static review is not sufficient, when PostgreSQL callback behavior must
  be verified, or when the user explicitly asks for tests.
- The primary validation target is PG18. When tests are necessary, prefer the
  narrowest PG18-focused command that covers the touched behavior, for example:
  - focused `cargo test ...`
  - focused or full `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- PG17 is optional compatibility coverage. Do not run PG17 tests unless the
  user explicitly requests PG17 validation or the change is specifically
  PG17-facing.
- Commit each reviewed code checkpoint. If tests are skipped under this policy,
  state that clearly in the commit/review context.
- After a checkpoint, add or update the matching review request in `review/` and commit that review-packet update separately.
- Push committed checkpoints and review-packet updates to the remote immediately after committing. Feedback that exists only locally is invisible to other agents.

## Reviewer Workflow

- When leaving feedback on a branch, commit to **that branch** and push immediately.
- If reviewing multiple branches, commit and push feedback to each branch separately.
- Never run destructive git operations (reset, rebase, drop commits) without reading the affected commits and getting explicit confirmation from the user first.
- After pushing, verify the push succeeded before moving on.

## Local Safety Rules

- Do not revert unrelated local changes.
- Preserve the current on-disk layout unless a very small change is clearly justified.
- Do not use `/tmp`-based hacks or alternate scratch homes to work around approval, sandbox, or environment constraints; use the normal repo and user tool layouts instead.
- Add ADRs for design decisions that need durable rationale.
