# Task 78 Review: RaBitQ Bounded Candidate Cutoff

## Summary

This packet requests review of commit `7a1ff3a3bdc6fcf8289207df7aa4ad5d3625be19`.

The change targets the Task 78 primary latency hypothesis: SPIRE is spending too much time scoring candidates that cannot survive the bounded candidate heap. The primary/default path is RaBitQ; TurboQuant remains unchanged as a comparison format.

## Change

- Adds a RaBitQ cutoff scoring API to `SpirePreparedAssignmentScorer`.
- Uses RaBitQ's scalar upper-bound pre-prune once the bounded V2 leaf candidate accumulator is full.
- Preserves the existing batch path for unbounded scans and for TurboQuant.
- Records pruned candidates through the existing truncated-candidate diagnostics path.
- Aligns RaBitQ scalar and batch scorer output on `estimate_ip_scalar_only`.

## Validation

- `artifacts/cargo-fmt-check.log`
  - `cargo fmt --all -- --check`
  - exit 0
- `artifacts/cargo-test-assignment-scorer.log`
  - `cargo test -p ecaz --no-default-features --features pg18 assignment_scorer -- --nocapture`
  - `9 passed; 0 failed`
- `artifacts/cargo-clippy-pg18.log`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - exit 0

## Notes

This is a first Task 78 slice. It does not yet include benchmark-suite evidence for end-to-end RaBitQ latency or TurboQuant comparison; that should be the next packet after review of the candidate-pruning semantics.
