# Review Request: SPIRE Candidate Tie-Break Contract

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `991b52da Pin SPIRE candidate tie breaks`

## Summary

This checkpoint addresses the remaining candidate-ordering follow-up from the
holistic SPIRE architecture review. It keeps relation-backed persistence
blocked, but pins the deterministic same-score merge contract before boundary
replicas, replacement epochs, or remote candidate merge add more candidate
sources.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`
- `spec/functional/FR-040-spire-routing-and-search.md`

## What Changed

- `SpireRoutedLeafScanRows` now carries the serving epoch used to build scored
  candidates.
- `SpireScoredScanCandidate` now carries serving epoch and assignment flags.
- Candidate ordering now ranks lower ORDER BY score first, then newer serving
  epoch, then primary assignment before boundary replica within the same epoch,
  then heap TID, PID, row index, and `vec_id` bytes.
- Added a focused comparator test for the newer-epoch and primary-vs-replica
  tie-break contract.
- Updated the Task 30 plan, architecture response, and FR-040 acceptance
  criteria with the same ordering contract.

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable `imports_granularity` and `group_imports` settings.
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `182 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

## Notes For Reviewer

- Phase 1 still filters boundary replicas out of visible primary scans. This
  checkpoint pre-wires the candidate ordering contract for the later dedupe and
  merge paths.
- The untracked architecture-review feedback file
  `review/30219-spire-foundation-progress-status/feedback.md` remains local and
  was not staged or committed by this checkpoint.
