# Review Request: SPIRE Recursive Epoch Publish Bundle

Head SHA: `405e46a1`

## Summary

Recursive routing epoch drafts now expose the same publish-bundle boundary used
by single-level build drafts.

The new methods encode:

- epoch manifest;
- object manifest;
- placement directory; and
- root/control state from caller-provided manifest locators.

Unlike single-level drafts, recursive epoch drafts do not own the local vector
allocator cursor. The methods require `next_local_vec_seq` from the coordinator
draft, keeping allocator ownership explicit for the upcoming relation publisher.

## Files

- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 25 passed, including `recursive_epoch_draft_encodes_publish_bundle_with_allocator_cursor`.
- `git diff --check`

No PG18 SQL test was run for this pure publish-helper bridge.

## Review Focus

- Confirm the explicit `next_local_vec_seq` argument is the right API shape for
  recursive epoch publish, given that the epoch draft owns next PID but not the
  local vector allocator cursor.
- Confirm the recursive draft should reuse the same publish coordinator
  validation path as single-level builds before relation publication is wired.
