# Task 66 packet 004: pairwise batched kernels

## Summary

This packet addresses reviewer flag B1 from packet 001. The bits=1 and bits=8
batch paths now use pairwise cross-candidate NEON kernels:

- `sum_query_dequant_neon_bits1_pair`
- `sum_query_dequant_neon_bits8_pair`

Each pair kernel processes two candidates inside one inner loop and reuses the
same query loads across both candidates. Odd batch tails still fall back to the
single-candidate NEON kernel.

## Validation

- `cargo test --lib --no-default-features --features pg18 quant::rabitq`
  - `41 passed; 0 failed`
- `cargo check --no-default-features --features pg18`
  - passed
- Criterion M5 batch measurement:
  - `bits1_batch1000`: `85.877 us` -> `70.664 us`
  - `bits8_batch1000`: `124.17 us` -> `112.68 us`
  - `bits8c3_batch1000`: `123.24 us` -> `113.00 us`
  - `bits8c4_batch1000`: `122.47 us` -> `112.70 us`

## Review Notes

This closes the structural issue: the batch kernels are no longer a plain
loop-of-single-candidate kernels. The measured M5 improvement is material but
not a 2x speedup versus packet 001's already-NEON batch path, so the packet is
explicit about the remaining performance-gate interpretation risk.
