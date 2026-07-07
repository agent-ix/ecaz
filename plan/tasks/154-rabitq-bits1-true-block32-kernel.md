# Task 154: true 32-wide RaBitQ bits=1 block kernel (transposed popcount / FastScan shape)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P2 (gate on Task 152 attribution)

## Why

The bits=1 "block32" kernel is not actually a 32-wide kernel:
`score_block32_neon_impl` loops `BLOCK_WIDTH/2` times calling the production
2-wide pair primitive (`src/quant/rabitq32/neon.rs:125-131`), and the AVX2
variant does the same (`src/quant/rabitq32/avx2.rs:46-52`). The "32" is a
batching granularity, not a SIMD width. A genuine 32-candidate transposed
sign-bit kernel (the classic RaBitQ/FastScan batch shape — codes transposed so
one register op covers many candidates per dimension group) has never been
attempted in this codebase. The IVF SoA scratch already presents contiguous
256-posting slabs (`src/am/ec_ivf/scan.rs:368-481`), which is exactly the
input such a kernel wants; the pointer-array kernel signature
(`codes: &[&[u8]; 32]`, `avx2.rs:46`) does not require contiguity but a slab
variant can exploit it.

Prior art to respect: the multi-bit transpose exists (`mb_neon` 4-wide /
`mb_avx2` 8-wide), and the bits=4 block-vs-pair routing was decided on
measurement (`src/am/ec_ivf/quantizer.rs:632-639`) — this task must win on
measurement the same way, per arch.

## Scope

- Implement a transposed 32-wide bits=1 kernel for NEON and AVX2 behind the
  existing rabitq32 dispatch (partial-width convention preserved per Task 93).
- Differential-test against the pair path (byte-equal estimates required —
  same arithmetic, different schedule).
- Microbench ns/candidate vs the current 2-wide loop per arch, then end-to-end
  A/B on IVF RaBitQ bits=1 at 10k/50k/100k. Keep the pair-loop fallback and
  route per-arch by measurement, exactly like the bits=4 precedent.

## Out of Scope (hard)

- No on-disk format change (transposition happens at scratch-fill or in
  registers, not in the stored payload — a stored-transposed format is a
  separate, bigger decision).
- No SVE/AVX-512 lanes (ADR-077 dispatch decisions stand).

## Gate / Exit Criteria

- Byte-equal recall, microbench + 10k/50k/100k A/B per arch, and a
  route-by-measurement dispatch decision recorded (win → routed in; loss → the
  negative recorded next to the bits=4 precedent and the kernel left
  unrouted). Task 152's attribution should confirm the bits=1 scorer share is
  worth the effort before implementation starts.
