# Task 65b Packet 006: Concurrency Model Tests

## Scope

This packet reviews commit `488610ef1dfb7fe698f3c25030a9264ee7f96142`, which closes the Slice E concurrency-test surface called out by the approved locking design feedback.

The code change extracts the ordered reducer commit body into a private `commit_vamana_pivot_proposal` helper and adds focused tests for the deterministic epoch proposal model:

- epoch proposal reads use the immutable epoch snapshot, not live reducer state committed earlier in the same epoch;
- same-epoch candidate reads are still counted by stale-read instrumentation;
- proposal completion order does not affect the final reducer output because proposals are sorted by pivot ordinal before commit;
- `batch_size = 1` parallel epochs produce exact serial adjacency output, preserving the byte-shape oracle before larger batches are tuned.

## Validation

Packet-local evidence is recorded in `artifacts/manifest.md`.

- `cargo test -p ecaz --lib --no-default-features --features pg18 task65b_`
  - `8 passed; 0 failed`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - passed

## Review Notes

This packet does not claim the full Task 65b acceptance gates. It specifically addresses the Slice E model-checking obligation from `reviews/task-65b/003-locking-design/feedback/2026-06-04-02-reviewer.md`.

Remaining work for full Task 65b still includes corpus-scale worker-zero byte equality, flush/batch/worker tuning, real10k and real100k performance gates, recall-gate evidence, and the final scaling-curve measurement packet.
