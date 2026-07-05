# Task 157: x86 software prefetch in RaBitQ AVX2/AVX-512 kernels (mirror the NEON lever)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

Task 66 banked software prefetch on the NEON side — the aarch64 kernels
prefetch +64B ahead via `prefetch_read_l1` (`src/quant/rabitq.rs:3951`; call
sites `:2414`, `:2508`, `:2731`, `:2805`). **No x86 RaBitQ kernel issues
`_mm_prefetch`** — the AVX2/AVX-512 bits=1/4/8 single, pair, and slab kernels
all stream codes with plain `_mm*_loadu`. The bits=1 path in particular was
characterized as bandwidth-bound in the Task 66 work; the same reasoning was
never applied to the Intel lane (Tasks 67/103 focused on arithmetic kernel
shape, not memory scheduling).

## Scope

- Add `_mm_prefetch` (T0, next-code-ahead — match the NEON distance, then
  tune) to the AVX2 and AVX-512 bits=1 pair/slab kernels first (the
  bandwidth-bound family), then bits=4/8 if the first measurement shows a win.
- Microbench ns/candidate on the Intel desktop, then end-to-end A/B on IVF
  RaBitQ at 10k/50k/100k (Intel lane).

## Out of Scope (hard)

- No prefetch in scalar paths, no NEON changes, no AVX-512 tier expansion
  beyond kernels that already exist (ADR-077 §8 stands).

## Gate / Exit Criteria

- Byte-identical recall (prefetch is semantics-free) and a measured Intel
  latency delta at 10k/50k/100k — win routed in, or an honest null recorded
  (prefetch dropped, finding kept). Closes on the A/B evidence.
