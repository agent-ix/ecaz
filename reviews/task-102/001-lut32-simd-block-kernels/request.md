# Task 102 Packet 001: lut32 SIMD Block Kernels

Code checkpoints under review:

- `3915c11a3` — `Add lut32 SIMD block kernels for AVX2, NEON, and SVE`
- `fc8db79af` — `Repack lut32 AVX2 kernel via byte transpose; add octet-granular tails`

## What Changed

The lut32 family (TurboQuant no-QJL 4-bit, the flagship 1536-dim lane) had
11-line scalar-delegating stubs for every ISA backend. This packet replaces
them with real kernels:

- **AVX2** (`src/quant/lut32/avx2.rs`): shuffle-repack register-LUT kernel.
  Code bytes are transposed from per-candidate rows into per-dim columns
  with a 3-pass unpack network once per 16-byte chunk, so the scoring loop
  is pure SIMD (no per-dim scalar nibble extraction). Each dim's 16-entry
  f32 LUT is selected by `permutevar8x32` over its two 8-float halves plus
  a blend on the index high bit. Lane count is octet-granular (8..=32), so
  sub-block tails pay only for the octets they occupy; single-lane tails
  take a scalar fast path.
- **NEON** (`neon.rs`): `vqtbl4q_u8` four-register byte-table select over
  the 64-byte per-dim LUT, byte indexes derived in-vector from the packed
  nibbles. Real kernel, compile-gated off x86; first hardware validation
  lands with the G4 pass.
- **SVE** (`sve.rs`): VL-agnostic `global_asm` helper that gathers LUT
  entries with an `ld1w` vector gather and accumulates in place. `Sve2`
  and `Sve` reported distinctly, Apple-aarch64 gated, per-family `cntw`
  helper per the grouped-PQ/qjl32 convention.
- Partial path (`mod.rs`): scalar hosts and single-lane tails score
  directly; AVX2 tails go through `ceil(live/8)` octets; other SIMD hosts
  keep the Task 101 padded-block path.
- All kernels preserve per-lane dim-order accumulation: **bit-exact** with
  the scalar block reference (anchor menu choice: bit-exact, no ADR-076
  tolerance needed). The surface-counter test in `candidate_batch` became
  host-conditional now that SIMD hosts attribute all candidates to kernel
  rows.

## The v1 lesson (why the rewrite happened inside this packet)

The first AVX2 shape extracted nibbles scalar per dim into a stack array.
Measured on the SPIRE full-block lane it was **slower than the scalar
block kernel** (1,371 vs 1,054 ns/candidate) — store-to-load forwarding
stalls on every octet. The shuffle-repack rewrite (`fc8db79af`) is the
shape the task brief recommends; both measurement passes are recorded in
the manifest so the negative result stays on the record.

## Evidence (artifacts/, manifest.md is the source of truth)

Two release-backend passes of the same 8-step suite (recall + latency,
kernel-on/off, HNSW exact-mode full_lut + SPIRE, real DBpedia 10k at
1536 dims), one at the pre-Task-102 baseline `50a86029c`, one at
`fc8db79af`. Backend provenance for both passes: `ecaz dev install`
SHA-asserted release build, PG restart, `ecaz_build_profile()` probe,
suite-manifest preflight record. No pg_test ran between install and bench.

Headline results:

| Gate | Result |
| --- | --- |
| Kernel rate, SPIRE full blocks | 1,054 → 235 ns/candidate = **4.5×** (target ≥2×, floor 1.5×) |
| Kernel rate vs same-head unbatched scalar | 1,313 → 235 ns/candidate = 5.6× |
| HNSW exact-mode batch-on p50 | 16.5 → 4.65 ms (ef=80, **−72%**); 27.1 → 6.91 ms (ef=160, **−74%**) |
| SPIRE batch-on p50 | 17.3 → 8.54 ms (**−50%**) |
| Recall | byte-equal at every measured cell |
| Direct rows | `quant=turboquant isa=avx2` kernel rows on both AMs; width histogram cited |

HNSW exact-mode batch-on now **beats batch-off** (4.65 vs 5.22 ms) — at
baseline it lost 3.3× because every sub-8 flush padded to a 32-lane
scalar block. The octet-granular tails plus the single-lane scalar fast
path are what closed that gap; the remaining `isa=scalar` rows are
exactly the 62,877 single-lane flushes, by design.

## Acceptance criteria status

| AC | Status |
| --- | --- |
| 1 — real kernels, stubs eliminated | AVX2 verified on hardware; NEON/SVE real but G4-pending |
| 2 — bit-exact or documented tolerance | Bit-exact (per-ISA parity tests incl. odd dims; transpose unit test) |
| 3 — scoring-share ≥2× target / 1.5× floor per ISA per AM | AVX2: SPIRE 4.5×, HNSW 2.8× per live candidate ✓; ARM ISAs deferred to G4 |
| 4 — recall byte-equal | Met at every measured cell |
| 5 — end-to-end no regression | Large wins on kernel-on cells; kernel-off cells unchanged within noise |
| 6 — real-ISA counter rows + width histogram | Met (see manifest tables) |

## Review focus

1. The AVX2 transpose network correctness argument (unit test +
   bit-exact parity at 1536/odd dims) and the octet-granular tail
   dispatch in `score_lut_no_qjl_4bit_partial`.
2. The NEON byte-index construction (`nibble*4*0x01010101 + 0x03020100`
   feeding `vqtbl4q_u8`) — review for G4 readiness; it cannot be executed
   locally (no aarch64 cross C toolchain for the full crate).
3. The SVE choice: `ld1w` vector gather rather than a literal `tbl` — a
   VL-agnostic 64-byte register table needs VL-dependent segment handling,
   so the gather is the portable SVE idiom; the G4 pass can evaluate a
   VL=128-pinned `tbl` variant if its share gate misses. Same asm-helper
   convention as grouped-PQ/qjl32.
4. Whether the v1 negative result and rewrite are adequately recorded
   (manifest "Interim v1 measurement" section).

Remaining for Task 102 closeout after this packet: G4 NEON/SVE2 evidence
(runs with the epic's deferred ARM pass; the Task 94/97 runbooks apply).
