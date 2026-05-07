# Review Request: SPIRE Object Reader Scan/Update Helpers

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `663362e9 Generalize SPIRE scan object reads`

## Summary

This checkpoint addresses the reader-abstraction follow-up from the first
holistic SPIRE architecture review. It keeps persistence unwired, but moves the
scan hot path and one read-only delta-update helper behind the shared
`SpireObjectReader` contract so future buffer-cache readers can share the same
call surface as the in-memory local object store.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Replaced concrete `&SpireLocalObjectStore` scan-helper parameters with
  `&impl SpireObjectReader`.
- Generalized read-only delta-update assignment vec_id collection to consume
  `SpireObjectReader`.
- Left mutating build/update draft helpers on the concrete local store where
  they still insert in-memory partition objects.
- Updated Task 30 and the architecture-feedback response note to record that
  scan helpers and read-only update collection now consume the reader trait.

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable `imports_granularity` and `group_imports` settings.
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

## Notes For Reviewer

- This is intentionally not relation-backed persistence.
- The untracked architecture-review feedback file
  `review/30219-spire-foundation-progress-status/feedback.md` remains local and
  was not staged or committed by this checkpoint.
