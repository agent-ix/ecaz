# Task 67 Review Request: X86 Scaffold Safety Follow-Up

## Summary

This packet tightens the Task 67 test-facing RaBitQ kernel scaffold added in
packet 011. The scaffold's `unsafe fn` wrappers now each document their
`# Safety` contract and use explicit inner unsafe blocks with local `SAFETY`
comments when calling the underlying target-feature kernels.

## Code Under Review

- Commit: `2a282430b7b22303b0566edfe532668d28e2c5e2`
- File: `src/quant/rabitq.rs`

## Behavior Added

- No runtime behavior change.
- Documents safety preconditions for scalar, AVX2, AVX-512, and gated AVX-512
  BF16 test scaffold wrappers.
- Makes the wrapper-to-kernel unsafe calls explicit, keeping the test scaffold
  aligned with Task 67's unsafe review gate.

## Validation

See `artifacts/validation.log` and `artifacts/manifest.md`.

- `cargo fmt` passed.
- `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_registers_expected_kernels --no-run` passed.
- `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available --no-run` passed.
- `git diff --check` passed.

This packet does not claim runtime execution, recall, benchmark, or AVX-512
hardware acceptance. Those remain part of Task 67's Intel Slice J gate.
