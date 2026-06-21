---
task: 118
packet: reviews/task-118/006-final-attribution-matrix
checkpoint_sha: 1a6e75720b5f10b8999a1d94958e99be39df2eff
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Compressed-Build Prefix Fix

## Summary

The initial 10k attribution suite pass found that the Task 118 compressed-build
load prefixes for TurboQuant and PqFastScan generated PostgreSQL identifiers
that exceeded the identifier limit. This checkpoint shortens only those
compressed-build prefixes in
`crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`.

The shortened prefixes are:

- TurboQuant: `task118_r{scale}_tq_cb`
- PqFastScan: `task118_r{scale}_pq_cb`
- RaBitQ: `task118_r{scale}_rq_cb`

Step names and artifact filenames remain descriptive; only database object
prefixes are shortened.

## Validation

- `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Artifact: `artifacts/cargo-test-ecaz-cli-hnsw-prefix-fix.log`
  - Result: `21 passed; 0 failed; 394 filtered out`
- Focused dry-run of the 10k compressed-build load steps:
  - Artifact: `artifacts/suite-dry-run-10k-compressed-prefix-fix.log`
  - Confirms load prefixes expand to `task118_r10k_tq_cb`, `task118_r10k_pq_cb`, and `task118_r10k_rq_cb`.

## Notes

The larger final attribution matrix is still in progress in this same packet.
This request is for the config checkpoint that unblocks the compressed-build
A/B lanes.
