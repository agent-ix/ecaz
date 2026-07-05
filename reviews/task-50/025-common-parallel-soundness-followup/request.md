# Task 50 Review Request: Common Parallel Soundness Follow-Up

## Summary

This packet addresses reviewer feedback from packet 024:

- `reviews/task-50/024-common-parallel-checked-helper-unsafe/feedback/2026-05-20-01-reviewer.md`

Code commit:

- `e861fe44 Address common parallel unsafe helper feedback`

The change removes the safe raw-pointer-input helper shape introduced in packet 024. Raw PostgreSQL pointers are converted to references at explicit unsafe callback boundaries, and internal descriptor helpers now take references or return references tied to validated `ParallelScanAttachment` state.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Packet 024 count | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/common/parallel.rs` | 63 | 44 | 38 | <=44 | met |

## Review Notes

- `validate_parallel_scan_state` now validates an `&EcParallelScanState`, so callers acknowledge raw pointer validity when creating the reference.
- `coordinator_ptr`, `worker_slots_ptr`, `reset_parallel_scan_layout`, and `initialize_parallel_scan_state` now take Rust references instead of raw state pointers.
- `ParallelScanAttachment::worker_slot` now returns a slot reference scoped to the attachment instead of returning a raw pointer through a safe method.
- Parallel worker slot claim/release/publish/snapshot behavior is unchanged.

## Validation

- `make unsafe-block-count PATHS='src/am/common/parallel.rs'` passed with count 38.
- `rustfmt --edition 2021 --check src/am/common/parallel.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes unsafe contract shape only; descriptor layout, atomics, callback wiring, and scheduling behavior are unchanged.
