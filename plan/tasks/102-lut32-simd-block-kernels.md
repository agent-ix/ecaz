# Task 102: lut32 SIMD Block Kernels (TQ no-QJL 4-bit, All ISAs)

Status: proposed (2026-06-10; in-epic per operator decision — ALL
optimization work lands inside the unify/batching epic)
Owner: coder (to be assigned; the Task 94/97 author is the natural
fit — same candidate-parallel kernel shape as the qjl32 transpose and
the F8 repack). Phase III.
Priority: 1 (flagship-lane kernel; blocks honest G4 evidence)

## Why

The lut32 family scores the canonical production lane — TurboQuant
no-QJL 4-bit at dim 1536, the storage format of every standard
DBpedia fixture — and it is the only kernel family with **no real
SIMD backend on any ISA**: `lut32/{neon,sve,avx2}.rs` are 11-line
scalar-delegating stubs (verified 2026-06-10). The "kernel" wins
measured since Task 87 on this lane are batch-amortization only.
Every lut32 counter row on Graviton 4 would read `isa=scalar`, and
Task 101's sub-width dispatch would route into scalar stubs.

Nobody owned this: Task 92 backfilled the module layout with stubs
and deferred G4 evidence to "the first real SVE2 backend" from Tasks
93–98, each of which built a different quant family. Source of
record: `reviews/task-99/000-pre-closeout-architecture-review/`
(seq 04).

## Scope

### In scope

1. **AVX2 kernel**: candidate-parallel (one lane per candidate, dims
   sequential), 16-entry LUT row per dim. Prefer
   `_mm256_shuffle_epi8`-style register-resident LUT (sharing the
   Task 94 F8 repack approach) over gather — F8's measurement showed
   gather-based LUT kernels run ~1.1× scalar.
2. **NEON kernel**: `vqtbl1q_u8` sibling, same shape.
3. **SVE2 kernel**: VL-agnostic `tbl` sibling, `Isa::Sve2` reported
   distinctly, Apple-aarch64 gated, `cntw` helper reused.
4. **Octet/partial entry points** compatible with the Task 101
   width-cascade driver, so SPIRE/HNSW lut32 tails get SIMD coverage.
5. **Local evidence**: bit-exact or ADR-076-tolerance parity per the
   established anchor menu; kernel-on/off suite cells on the standard
   1536-dim fixtures (recall byte-equal, direct
   `quant=turboquant` kernel rows under real ISAs, scoring-share
   ladder per the Task 93/94 gates).

### Out of scope

- QJL lanes (qjl32, Task 97 — done), grouped-PQ (Task 94 F8), and
  all other families.
- AVX-512 (remains the deliberately deferred tier, post-Task-99).
- Dispatch-layer changes (Task 101 owns the driver).

## Acceptance criteria

1. `lut32/{avx2,neon,sve}.rs` are real kernels returning their true
   dispatched ISA; stubs eliminated.
2. Bit-exact parity with the scalar block reference (LUT accumulation
   preserves per-candidate dim order), or documented ADR-076
   tolerance with the forced-scalar anchor — state which, per the
   anchor menu.
3. Scoring-share ≥2× target / 1.5× floor per ISA per AM on the
   standard fixtures (the lane's LUT is L1-resident at 1536d per
   ADR-025 — headroom should be real).
4. Recall byte-equal at every measured cell.
5. End-to-end no regression beyond noise.
6. Counter rows attribute real ISAs; flush-width histogram cited for
   tail coverage once Task 101 lands.

## Sequencing

In-epic. The Graviton 4 evidence passes (Tasks 94/97 runbooks + the
Task 99 profile) run after Task 101 + Task 94 F8 + this task, so ARM
evidence covers final kernel shapes for every family including the
flagship lane.

## References

- reviews/task-99/000-pre-closeout-architecture-review/ (seq 04)
- Task 94 F8 slice (shuffle-repack approach to share)
- Task 97 packets 016/018 (candidate-parallel + octet precedent)
- ADR-025 (LUT cache analysis), ADR-076, docs/block-kernel-development.md
