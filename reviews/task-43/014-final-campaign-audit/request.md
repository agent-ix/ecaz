# Task 43 Final Campaign Audit Review Request

## Scope

This packet closes G8 for Task 43. It does not add production code. It updates
the campaign tracker and supplies the final aggregate evidence that the Task 43
Miri/cargo-careful safety campaign is complete on this branch, pending external
review.

Validation head: `c44d0bccb9647a1b50c14f3b68f7fb857c763126`

## Evidence

- `artifacts/cargo-fmt-check.log`: `cargo fmt --all -- --check`, exit 0.
- `artifacts/git-diff-check.log`: `git diff --check`, exit 0.
- `artifacts/careful-harness-cargo-test.log`: 69 passed, 0 failed.
- `artifacts/make-careful.log`: 69 passed, 0 failed; doctests 0 passed, 0 failed.
- `artifacts/make-miri-expanded.log`: 87 passed, 0 failed.
- `artifacts/make-miri-tree.log`: 87 passed, 0 failed.
- `artifacts/make-miri-many-seeds.log`: exit 0; 128 distinct seed attempts
  covering seeds 0 through 127. The aggregate run includes the real threaded
  common-parallel test added in packet 007.
- `artifacts/manifest.md`: command metadata and key result lines.

## Reviewer Finding Disposition

The tracker at
`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
is updated as the completion source of truth.

- Many-seeds structural emptiness: closed by packet 007 plus this packet's
  aggregate `make miri-many-seeds` run.
- Strategy breadth gaps: closed by packets 008-011.
- Row-independent remote typed payload parser: closed by packet 009.
- SPIRE vacuum/delete-delta visibility: closed by packet 010.
- cargo-careful mirroring: closed for all path-liftable surfaces by packet 012;
  SPIRE mirrors remain explicitly blocked on extraction or a SPIRE careful
  micro-harness.
- Mutation/sensitivity probes: closed by packet 013 with nine failing temporary
  mutations and restored source state.
- Final audit: closed here by mapping all gates and reviewer findings to
  packet-local artifacts.

## Review Focus

Please verify that:

- G8 should now be marked Done.
- The final aggregate lanes are sufficient evidence for campaign closeout.
- The tracker still honestly preserves the SPIRE careful blockers and
  non-exhaustive mutation caveats rather than overclaiming.
