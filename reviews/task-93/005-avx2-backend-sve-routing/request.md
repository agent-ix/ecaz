# Task 93 Packet 005: AVX2 Backend + SVE-to-NEON Routing (code slice)

Code-only slice for Phase D (AVX2) plus an interim Phase C decision.
Measurement evidence is deliberately deferred to the hosts that can produce
it (see §Deferred below).

## Commit under review

- `2726b8b4a` (rebased to `4872107d8` lineage) — AVX2 backend + SVE routing.

## AVX2 backend (`src/quant/rabitq32/avx2.rs`)

Mirrors the approved NEON backend structure exactly:

- block32 and partial (1..=31) entry points reuse the production AVX2
  byte-LUT pair/single primitives
  (`rabitq::sum_query_dequant_avx2_bits1{,_pair}`, made `pub(crate)`;
  their `# Safety` docs already existed). Operation orders are identical to
  the production `estimate_ip_bits1_batch` AVX2 path, so kernel scores are
  bit-equal with production batch scoring on AVX2 hosts by construction —
  the same tight-anchor argument packet 003 approved for NEON.
- Runtime `avx2`+`fma` detection; `Isa::Avx2` reported only when the AVX2
  impl actually ran; non-x86_64 builds compile the scalar fallback.
- New `unsafe` impls carry `# Safety` docs covering feature detection and
  shape/length invariants (validated by the batch wrapper before dispatch).

## SVE dispatch decision (interim, Phase C pending)

`Isa::Sve`/`Isa::Sve2` dispatch (block and partial) now routes through the
validated NEON backend instead of the scalar fallback: every SVE host
implements NEON, so degrading to forced-scalar on Graviton-class hosts
would reproduce exactly the regression packet 004 fixed for sub-32 batches.
`Isa::Sve`/`Sve2` are never reported until a real SVE kernel runs (counter
attribution stays truthful: such hosts publish `isa=neon` rows). The real
vector-length-agnostic SVE kernel remains the Phase C Graviton deliverable.

## Tests

- ISA expectations generalized via `host_expected_simd_isa()` (NEON on
  aarch64, AVX2+FMA on x86_64, scalar otherwise).
- `simd_block32_is_bit_equal_with_production_batch` and the partial-dispatch
  test now prove kernel ≡ production-batch bit-equality on whichever SIMD
  host runs them.

## Validation

- M5/NEON (this host): clippy `-D warnings` clean; rabitq32 6/6,
  candidate_batch 10/10 (logs in `artifacts/`). The dispatch refactor is
  fully exercised here through the NEON paths.

## Deferred (explicitly, with owners)

- **AVX2 compile + runtime + bench evidence → Intel lane.** This host
  cannot execute x86 code; a local cross-`cargo check` is additionally
  blocked by the Homebrew-rust/rustup-std toolchain split (documented in
  the commit message; not worked around per repo env rules). The AVX2
  backend calls only production primitives that already compile and ship
  on x86_64. Phase D measurement (Intel desktop) should run the same
  suite shape as packets 003/004 plus `cargo test rabitq32` on that host.
- **SVE kernel + Graviton 4 measurement → Phase C** (AWS lane, pending
  authorization), including measured runtime vector length and `isa=sve2`
  counter rows per the packet-001 design.

## Review request

Please review the AVX2 backend (primitive reuse, safety boundary, ISA
attribution), the interim SVE→NEON routing decision, and the deferral
framing. Next packet: per-(AM × ISA) closeout matrix over the measured
cells.
