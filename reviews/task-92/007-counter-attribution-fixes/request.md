# Task 92 Packet 007: Counter Attribution Fixes

## Summary

This packet addresses the reviewer request-changes feedback on
`reviews/task-92/003-counter-surface-phase2`.

Code checkpoint:
`4c7c9bab824eea9ace420ad96175aa616f548408`

Changes:

- Split block counter recording into explicit kernel and scalar-tail calls so a
  single elapsed interval is not attributed to both lanes.
- Force scalar-tail attribution through `isa=Scalar`; kernel and scalar-tail
  rows are no longer mixed through one API call.
- Preserve Task 87 compatibility aggregation while deriving `lut32_*` fields
  from kernel-only flush and candidate counters.
- Keep the current LUT32 scalar fallback row under `isa=Scalar`; future
  Graviton 4 SVE2 kernels can report kernel blocks under `isa=Sve2` while
  their scalar tails remain in `isa=Scalar`.

## Validation

- `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
  - `5 passed; 0 failed`
  - artifact: `artifacts/cargo-test-candidate-batch.log`
- `cargo test -p ecaz-cli commands::bench::tests --no-default-features`
  - `7 passed; 0 failed`
  - artifact: `artifacts/cargo-test-bench-module.log`
- `git diff --check`
  - passed with no output
  - artifact: `artifacts/git-diff-check.log`

## Reviewer Notes

The new unit coverage includes a synthetic `isa=Sve2` kernel record plus a
scalar tail record. It asserts that the SVE2 row contains only kernel fields and
the Scalar row contains only scalar-tail fields, which matches the Graviton 4
rollout contract.
