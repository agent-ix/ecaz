---
task: 92
packet: 004-counter-cli-output
agent: coder
date: 2026-06-09
---

# Task 92 Phase 2: Block Kernel Counter CLI Output

## Summary

This checkpoint extends the bench CLI counter collection/output path to prefer
the new block-kernel counter surface while keeping Task 87 output during the
transition.

Code commit:

- `f70b4e5a6193ed25e6b3144589794c92ff78daeb`
  `Emit block kernel counter lines in bench CLI`

Changes:

- Adds CLI-side `BlockKernelCounterSnapshot` and
  `BlockKernelCounterSnapshots`.
- `--task87-candidate-batch-counters` now resets through
  `ec_block_kernel_scoring_reset()` when available, falling back to the Task 87
  reset function for older extension binaries.
- Latency and SPIRE pipeline benchmarks now snapshot
  `ec_block_kernel_scoring_snapshot()` and format `[block-kernel-counters]`
  lines.
- The CLI still collects and emits `[task87-counters]` lines in the same output
  so existing packet parsers remain compatible during this implementation
  slice.
- If the extension binary does not yet expose the new snapshot function, the
  CLI emits the Task 87 compatibility lines only.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `git diff --check`: passed with no output.
- `cargo test -p ecaz-cli commands::bench::tests --no-default-features`:
  `7 passed; 0 failed`.

## Review Focus

- Confirm emitting both `[block-kernel-counters]` and `[task87-counters]` is the
  right transition behavior for Phase 2.
- Confirm keeping the existing `--task87-candidate-batch-counters` CLI flag name
  is acceptable for this compatibility slice.
- Confirm the line format matches the Phase 1 counter contract.
