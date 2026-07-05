# Task 155: RaBitQ NEON bits=4 pair/batch kernel (close the ARM batch gap)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

`estimate_ip_batch_impl` has NEON arms only for bits=1 and bits=8
(`src/quant/rabitq.rs:4459-4478`); `NeonBits4`/`NeonBf16Bits4` fall through to
`estimate_ip_batch_scalar` (`:4492-4503`), which loops candidates through
`estimate_ip_scalar_only_from_sum_context` — re-running kernel selection **per
candidate** and forgoing 2-candidate ILP. The x86 side has bits=4 pair kernels
(`avx2_bits4_pair:3715`, `avx512_bits4_pair:3610`); ARM does not. IVF routes
bits=4/8 through this batch path by measured decision
(`src/am/ec_ivf/quantizer.rs:632-651`), so every rabitq4 batch on M5/Graviton
pays the per-candidate dispatch today.

Distinct from the rejected finding: Task 93 measured the bits=4 **block
transpose** 2.8x slower than the per-candidate arithmetic estimator on M5
NEON. A **pair** kernel (2 candidates interleaved, same arithmetic shape as
the shipped NEON bits=1/bits=8 pairs) was never tried.

## Scope

- Implement `neon_bits4_pair` + the `estimate_ip_batch_neon_bits4` slab
  wrapper, mirroring the existing NEON bits=8 pair structure; hoist kernel
  selection to once per slab.
- Differential-test byte-equality against the single NEON bits=4 kernel.
- Microbench + end-to-end A/B on IVF rabitq4 at 10k/50k/100k on an ARM host
  (M5 local is sufficient; Graviton optional).

## Out of Scope (hard)

- No block-transpose revisit (measured negative, Task 93). No bf16 scope
  creep (Task 66 Slice F owns that gate).

## Gate / Exit Criteria

- Byte-equal recall and a measured ARM bits=4 batch win (or an honest
  negative recorded next to the Task 93 block-transpose datum). Closes on the
  A/B evidence.
