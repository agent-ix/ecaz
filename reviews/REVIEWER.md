# Reviewer Playbook

Full rules for the reviewer role. `AGENTS.md` carries the shared
task-scoped packet/feedback format and points here. Load this doc when acting
as reviewer.

## Trigger

Invoked when the user asks for code review, a phase closeout review, or review
of a specific task packet or branch.

Default scope when the user does not name a specific packet:

- Every packet in the owning task bucket under `reviews/task-{id}/` that lacks
  a reviewer feedback file since the last reviewer-feedback commit on this
  branch, plus
- Any uncommitted in-progress slice on the working tree that extends the same
  task.

If the user names a packet, review only that packet and any directly related
sibling packets that share the same finding.

## Scope of a Code Review

A code review covers, at minimum:

- **Code**: correctness, edge cases, error paths, panics, concurrency, resource
  lifetime.
- **Design**: alignment with `plan/design/*.md` notes, ADRs, and the task's
  design checkpoint.
- **Spec/contract**: alignment with the design's stated invariants, flag
  semantics, on-disk format claims, and SQL surface.
- **Tests**: per-error-path coverage on new validators/codecs, integration
  coverage for new SQL surfaces, regression coverage for prior bugs.
- **Measurement claims**: if a packet has `artifacts/`, verify the numbers in
  `request.md` are supported by the raw artifacts and that lanes match across
  compared lanes.

Prefer correctness findings over style comments. Treat the current on-disk
layout as intentional unless a small concrete defect requires change.

## Output

- One feedback file per packet for localized findings, written at
  `reviews/task-{id}/{ordinal-topic}/feedback/{YYYY-MM-DD}-{seq}-reviewer.md`.
- For cross-cutting findings that span a whole phase, write per-packet files
  for the localized parts and put the cross-cutting summary in the closeout or
  most-recent packet for that task.
- If the work being reviewed is in-progress and no packet exists yet, create
  the expected packet folder under the owning task bucket and drop a
  "pre-checkpoint review" feedback file there. Do not wait for the coder to
  make the packet first.
- Any review, test, benchmark, or measurement logs cited by feedback must live
  under the packet's `artifacts/` directory.
- Summarize the same findings to the user in chat after the file is written.
  The coder is the priority audience; the user is secondary.

## Mechanics

- Commit feedback to the branch under review and push immediately per
  `AGENTS.md`'s Push and Visibility rule. Verify the push.
- If reviewing multiple branches, commit and push feedback to each branch
  separately.
