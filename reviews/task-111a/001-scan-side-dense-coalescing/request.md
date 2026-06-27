# Task 111a Review Request: Scan-Side Dense Coalescing

## Summary

This packet requests review for Task 111a Phase 1 / Approach A only.

The code adds scan-side cross-block coalescing for dense IVF posting blocks
behind the existing `dense_posting_blocks` gate. Dense blocks remain one page
on disk; the scan now accumulates consecutive dense postings from the same list
into scan-opaque scratch and flushes that scratch through the existing SoA batch
scorer up to `IVF_POSTING_SCRATCH_SOA_BATCH_POSTINGS`.

The implementation preserves the current deleted-bitmap filtering, live-TID
budgeting, heap-TID expansion, candidate dedup, and distance scoring semantics.
Row posting scratch and dense coalesced scratch are flushed separately, with
separate counters, so dense-only scans can be distinguished from row reshaping.

## Code Under Review

- `src/am/ec_ivf/scan.rs`
  - adds scan-opaque dense coalescing scratch lifecycle.
  - adds dense coalesced flush handling through the existing batch scorer.
  - flushes dense coalesced scratch at list boundaries, row-block boundaries,
    full scratch capacity, and end of scan.
- `src/am/ec_ivf/page.rs`
  - exposes `IvfDensePostingBlockRef::gamma(index)` for coalescing.
- `src/am/common/explain.rs`
  - adds EXPLAIN-visible dense coalescing counters.
- `src/tests/ec_ivf.rs`
  - updates focused dense posting block PG tests to assert the dense coalesced
    path is used while row scratch flushes stay zero for dense-only scans.

Code commit: `b47d5b78ccc6a67be3fec2af9da004551e6cb2c6`

## Validation

Artifacts are packet-local under `reviews/task-111a/001-scan-side-dense-coalescing/artifacts/`.
See `artifacts/manifest.md` for commands, timestamps, and key result lines.

- `cargo check -q --lib`
  - `artifacts/cargo-check-lib.log`
  - exited `0`.
- `cargo check -q --lib --features pg_test`
  - `artifacts/cargo-check-lib-pg-test.log`
  - exited `0`.
- `cargo test -q posting_scratch_soa --lib`
  - `artifacts/cargo-test-posting-scratch-soa.log`
  - `4 passed; 0 failed`.
- `cargo test -q ivf_explain_counters --lib`
  - `artifacts/cargo-test-ivf-explain-counters.log`
  - `1 passed; 0 failed`.
- `cargo pgrx test pg18 dense_posting_blocks`
  - `artifacts/cargo-pgrx-test-pg18-dense-posting-blocks.log`
  - `5 passed; 0 failed`.

## Not Done In This Packet

- Approach B on-disk multi-page dense packing is not implemented.
- No `ecaz bench suite` matrix has been run yet for 50k / 100k / 1M.
- No default-promotion decision is requested here.
- Task 111a remains active after this packet; this packet is only the first
  code checkpoint for Approach A correctness and observability.
