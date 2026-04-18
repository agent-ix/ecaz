---
id: ADR-039
title: "ARM SVE as Secondary SIMD Backend for ARM Targets"
status: PROPOSED
impact: Affects FR-014, NFR-001, ADR-006, ADR-030
date: 2026-04-18
---
# ADR-039: ARM SVE SIMD Strategy

## Context

tqvector's current ARM SIMD story is **NEON** (ADR-006, ADR-031). NEON is
the universal ARM64 baseline: fixed 128-bit vectors, present on every
aarch64 chip we care about, well-understood instruction set.

**SVE (Scalable Vector Extension)** and its successor **SVE2** are ARM's
newer SIMD ISA. Key differences from NEON:

- **Vector length is hardware-defined**, not fixed. Same binary runs on a
  chip with 128-bit vectors and a chip with 512-bit vectors; the code
  adapts at runtime via `svcntb()` and predicated operations.
- **Richer predication.** Every instruction can be masked by a predicate
  register, eliminating the branch/blend gymnastics NEON requires.
- **Gather/scatter support** for strided and indexed memory access,
  relevant to FastScan LUT lookups.
- **Higher peak throughput** on chips that implement wider vectors.

Hardware availability is spotty:

| Chip | NEON | SVE | SVE2 |
|---|---|---|---|
| Graviton 2 | Yes | No | No |
| Graviton 3 | Yes | Yes (256-bit) | No |
| Graviton 4 | Yes | Yes (256-bit) | Yes |
| Apple M1–M3 | Yes | No | No |
| Apple M4+ | Yes | Yes | Yes |
| Neoverse V1 | Yes | Yes (256-bit) | No |
| Neoverse V2 | Yes | Yes (128/256-bit) | Yes |
| Ampere Altra | Yes | No | No |

Public cloud ARM instance populations reflect the split: Graviton 2/3
still dominate deployed fleets; Graviton 4 adoption is growing but
incomplete. The "every ARM chip" ISA is NEON; SVE is "most modern
server ARM."

## Decision

**NEON remains the ARM baseline.** Every SIMD kernel must have a working
NEON implementation. SVE is a **secondary backend** selected at runtime
when the hardware supports it.

### Code organization posture

We write SVE as **vector-length-agnostic (VLA)** code, not as
per-length specializations. VLA is SVE's intended usage pattern: one
implementation serves 128-bit through 2048-bit hardware. Writing
separate 256-bit and 512-bit specializations doubles maintenance cost
and provides no portability benefit.

### Build-time posture

- SVE support is enabled under `cfg(target_feature = "sve")` guarded by
  runtime detection, not a compile-time hard requirement.
- CI builds a single aarch64 binary that contains both NEON and SVE
  kernels; dispatch happens at first use per quantizer/scanner.
- Compile-time `#[target_feature]` annotations isolate SVE intrinsics
  so non-SVE hardware still links.

### Kernel scope

SVE versions land for the three ARM-hot kernels, in order:

1. **FastScan LUT scoring** (`grouped_pq_score_*`) — primary win; SVE's
   predication and gather support suit the vpshufb analog.
2. **FWHT / SRHT rotation** — the rotation pass auto-vectorizes on
   NEON; SVE widens it on supporting chips.
3. **Binary sidecar POPCNT scoring** — SVE has wide POPCNT; potential
   2–4× throughput over NEON's `vcnt`.

Each kernel must benchmark neutral-or-better against NEON on at least
one SVE target before replacing the NEON path at dispatch time.

### What we don't adopt

- **SVE-only kernels without NEON fallback.** Would fragment the ARM
  build.
- **Per-vector-length specialization** (e.g., separate 256-bit and
  512-bit implementations). Violates SVE's design intent.
- **SVE2-specific intrinsics** unless clearly justified per kernel.
  SVE2 is a superset; baseline SVE coverage is the pragmatic target.

## Consequences

### CI matrix grows

Current ARM testing covers NEON on one or two chip generations. SVE
coverage requires at least Graviton 3 instances in CI (AWS m7g or
similar). Adds cost but bounded — we don't need Apple Silicon in CI
for this.

### Documentation and positioning

tqvector's "runs well on Graviton" claim becomes more defensible.
Graviton 4 + SVE2 + PqFastScan should measurably beat Graviton 2 +
NEON, which matters for the growing population of ARM Postgres
deployments.

### Migration / compatibility

No wire format impact. No index rebuild required. Users on non-SVE
ARM chips continue on NEON transparently.

### What SVE will not help

- **x86 targets.** Separate concern (AVX-512 as a task, not an ADR
  per this plan).
- **NEON-only chips (Graviton 2, Apple M1–M3).** They continue on the
  NEON path. No regression, no improvement.
- **Kernels that are not SIMD-bound.** FWHT on short vectors, metadata
  operations — SVE doesn't change their cost.

## Alternatives considered

### Stay on NEON indefinitely

Simplest. Defensible given that NEON runs everywhere. Gives up
2–4× throughput wins on Graviton 3+/Graviton 4 and future Apple
Silicon server use.

### Adopt SVE and drop NEON

Would simplify the codebase but breaks older ARM chips. Not
acceptable given current cloud fleet composition.

### Write per-length specializations

Fastest peak performance but doubles maintenance. Against SVE's
design philosophy. Rejected.

### Defer until Graviton 4 is the baseline

Reasonable. Would push this ADR out by 12–24 months until the
chip population shifts further. Trade-off: delays the scale-band
story on ARM in the interim.

## References

- ADR-006: Own quantizer implementation (SIMD target matrix)
- ADR-030: FastScan Grouped Subvector Scoring
- ADR-031: RaBitQ Binary Pre-Filter (NEON popcount notes)
- ARM Scalable Vector Extension (SVE) architecture reference
- AWS Graviton 3 / 4 SIMD capabilities documentation
