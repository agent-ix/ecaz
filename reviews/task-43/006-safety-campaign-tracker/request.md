# Review Request: Task 43 Safety Campaign Tracker

## Summary

This is a strategy and tracking correction, not an implementation packet.

The original Task 43 strategy packet did not establish a strict enough
completion contract for a serious Miri/cargo-careful safety campaign. This
packet installs a canonical tracker under the original strategy packet:

- `reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`

The tracker is now the source of truth for Task 43 completion. It records:

- campaign rules,
- non-negotiable completion gates,
- current evidence baseline,
- subsystem-by-subsystem coverage status,
- validation requirements,
- planned follow-up packets,
- reviewer feedback disposition,
- the rule that Task 43 is not complete until every row is done or precisely
  blocked with extraction work.

## Review Focus

- Confirm the tracker is strict enough for the user-stated bar: a serious,
  thorough, extensive safety campaign.
- Confirm all reviewer findings from packets 001 through 005 are represented.
- Confirm the tracker makes the current status unambiguous: **Task 43 is in
  progress, not complete**.

## Validation

Strategy-only packet. No code tests were run.

Static checks:

- Confirmed no leftover Miri process from the interrupted exploratory run.
- Read reviewer feedback for packets 001 through 005.
- Updated the original strategy packet to point at the canonical tracker.
