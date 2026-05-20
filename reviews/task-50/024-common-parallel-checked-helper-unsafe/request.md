# Task 50 Review Request: Common Parallel Checked Helper Unsafe Reduction

## Summary

This packet handles `src/am/common/parallel.rs`, the final top-15 file.

Code commit:

- `d77fcf39 Reduce common parallel unsafe blocks`

The change consolidates repeated unsafe regions around parallel scan descriptor validation, worker-slot pointer lookup, and AM-private parallel scan state initialization.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Packet count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/common/parallel.rs` | 63 | 44 | <=44 | met |

## Review Notes

- Descriptor sizing and MAXALIGN calculations are unchanged.
- Worker-slot capacity, claim, publish, release, snapshot, and rescan behavior are unchanged.
- Reviewer feedback in `feedback/2026-05-20-01-reviewer.md` requested a follow-up for the raw-pointer helper contract shape. That fix landed separately in packet 025.

## Validation

- `make unsafe-block-count PATHS='src/am/common/parallel.rs'` passed with count 44.
- `rustfmt --edition 2021 --check src/am/common/parallel.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes unsafe ownership structure but not parallel scan scheduling, slot atomics, descriptor layout, or PostgreSQL callback wiring.
