---
id: ADR-077
title: "Block Kernel Completeness Closing Record"
status: PROPOSED
impact: Closes the Task 87/91/92/93-98/101-104 kernel-completeness initiative; sets the operating convention for all future quantized scoring work, the per-family anchor-regime menu, the per-AM enablement policy, and the coverage gate for new quants.
date: 2026-06-11
---
# ADR-077: Block Kernel Completeness Closing Record

## Context

The kernel-completeness initiative consolidated all compressed-domain
scoring behind one architecture:

- **Task 87** — `CandidateBatch` plumbing + the first (scalar) lut32
  block scorer and per-AM counters.
- **Task 91** — the `QuantCodec` trait (ADR-071/072): one scoring
  interface, 8 registration sites, AMs never touch ISA modules.
- **Task 92** — kernel infrastructure (ADR-076): 32-wide block width,
  runtime ISA dispatch, `(surface × quant × isa)` counter matrix,
  per-family module layout, `kernel_status` markers.
- **Tasks 93–98** — per-quant kernel families: rabitq32, grouped-pq,
  hamming32, (96: stop condition — no 2-bit surface), qjl32,
  tiled_lut32/int8_approx32.
- **Task 101** — one generic width-cascade batch driver
  (32 → octet → partial → scalar) for all seven families;
  prevalidation everywhere; `TurboQuantTiledLut`/`TurboQuantInt8`
  counter kinds; DiskANN grouped-PQ SIMD coverage ~3% → full.
- **Task 102** — real lut32 SIMD kernels on the flagship lane
  (AVX2 4.5× on SPIRE full blocks; shuffle-repack shape).
- **Task 103** — Intel AVX2 column closure (int8_approx32 10.4×;
  rabitq32 validated; tiled_lut32 retired; hamming32 AVX2 skip).
- **Task 104** — Apple-silicon (M5) supported-target column: every
  family ≥1.5× floor or documented marker; candidate-parallel qjl32
  NEON kernel (0.83× → 3.5×); no-SVE dispatch ladder validated.
- **Task 99** — this closeout: the aggregate matrix
  (`reviews/task-99/001-aggregate-matrix/artifacts/cross-am-quant-isa-matrix.md`),
  the index × quant × mode profile (G4 + AWS-Intel lanes), and this
  record.

## Decisions

### 1. The block-kernel pattern is the default for compressed-domain scoring

Any new quantized scoring path ships as an ADR-076 kernel family: the
Task 92 module skeleton (`{mod,scalar,neon,sve,avx2}.rs`), dispatch
through the Task 101 width-cascade driver, `(surface × quant × isa)`
counter attribution, and suite-driven evidence with the established
gates (parity per the family's anchor regime, recall preservation,
direct counter rows, kernel-on/off end-to-end). Per-candidate scalar
scoring is no longer an acceptable shape for a new quant lane.

### 2. The aggregate matrix is the coverage gate

The (AM × quant × ISA) matrix is the project-level completeness gate.
Every cell is one of: **real** (measured, with source packet),
**retired**, **skip** (measured decision), **missing_kernel** (real
surface, no kernel — must carry a scope-decision citation), or
**structurally_absent** (no surface, with source evidence). A new quant
or a new AM adds a row/column and must fill or mark every cell before
its closeout. Suite steps assert markers via `kernel_status` tags
(including the runnable `retired` marker from Task 104).

### 3. Anchor/tolerance regime menu (resolves pre-closeout F5)

Four regimes are ratified; a new kernel family must pick one explicitly
in its design packet, in this order of preference:

1. **Integer-exact across backends** (int8_approx32, hamming32):
   integer accumulation is order-independent → strict `to_bits()`
   equality on every ISA. Choose whenever the family's algebra permits.
2. **Bit-exact scalar-order** (lut32, grouped_pq): float accumulation
   with dim-order preserved per lane; SIMD output bit-equal to the
   scalar reference. Choose when the LUT/accumulation order can be
   preserved without losing the kernel win.
3. **Forced-scalar anchor + tolerance dispatch pair** (qjl32): a
   bit-exact forced-scalar anchor plus a 4-ULP/1e-6 per-slice contract
   for reordered-FMA dispatch. Choose when reordering is required for
   performance. The Task 97 packet 015 diagnostic (5,920 ULP from
   cancellation under reordering) is the standing justification for
   why the anchor must be forced-scalar.
4. **Production-same-order bit-equality + envelope + recall binding**
   (rabitq32: 1e-5 envelope, measured 22 ULP/1.55e-6 at dim 1536).
   Choose when the kernel mirrors an existing production SIMD path
   whose order it preserves by construction.

Recall preservation at the bench level is binding in all four regimes;
byte-equal recall is the expected outcome everywhere it was measured
(40/40 pairs on M5; every local closeout cell).

### 4. Per-AM enablement policy (resolves pre-closeout F4)

- **SPIRE, HNSW: always-on.** Kernel-on won or tied at every measured
  cell; no GUC-off production posture exists (the GUCs remain as
  diagnostic A/B switches).
- **DiskANN: GUC default-on** (`ec_diskann.candidate_batch_scoring`).
  Measured parity-to-win; the off switch stays for diagnostics.
- **IVF: GUC default-off** (`ec_ivf.scratch_soa_batch_decode`) —
  **pending decision input**: batch-on trades away suffix-max cutoff
  pruning (Task 94 packet 024 F1), and the Task 101 release rerun
  measured batch-on winning all six IVF grouped-PQ cells (−3.8% to
  −10.4%) despite the trade. The Task 99 profile (batch on/off at every
  IVF cell, three lanes) is the dataset for the default flip decision.
  This ADR records the policy menu; the IVF default decision is taken
  when this record flips to ACCEPTED, citing the profile.

### 5. Counter-key attribution (resolves pre-closeout F2)

Every kernel family records under its own `QuantCodecKind` — Task 101
added `TurboQuantTiledLut` and `TurboQuantInt8`, so HNSW exact modes no
longer share the `TurboQuant` key. Mode disambiguation by step metadata
is no longer load-bearing. The Task 87 compat surface keys on
`TurboQuant` only, unchanged.

### 6. ISA dispatch policy

`select_highest_isa` prefers the highest available tier
(avx2 > sve2 > sve > neon > scalar). Two qualifications:

- A family whose higher-tier entry has no real kernel must decline
  (return None / route down) so attribution stays truthful — the
  rabitq32 NEON-routing-on-SVE-hosts behavior is the precedent.
- The `ecaz.isa_cap` session GUC (Task 99 packet 004) caps dispatch for
  per-ISA A/B on one host. The Graviton 4 NEON-capped pass exists
  because G4's SVE2 is 128-bit — the same width as NEON — so
  "SVE2 over NEON" is a per-family measurement, not an assumption.
  [To record at ACCEPT time: the per-family G4 SVE2-vs-NEON outcome
  from `t99-g4-neon-cap-suite.json`.]

### 7. Structural lessons (standing design facts)

1. **Batch SIMD trades against per-candidate pruning.** Block scoring
   forfeits early-exit bounds (IVF suffix-max). The trade is now
   measurable per cell via counters instead of assumed; any future
   batch path over a pruned scan must measure it.
2. **Batch SIMD trades against batch-formation width.** Graph AMs are
   structurally width-bound (greedy descent consumes each expansion's
   scores before forming the next), so partial/octet dispatch — not
   cross-expansion batching — is the coverage answer (Task 101).
   The flush-width histogram sizes kernel upside in advance.

### 8. Deliberate exclusions and honest bounds

- **AVX-512**: deliberately excluded tier; revisit post-Task-99 only
  with measurement justification.
- **Quantized-LUT (u8 fast-scan) lut32 variant**: deferred indefinitely
  (operator decision 2026-06-10) — breaks the byte-equal recall regime,
  post-Task-102 upside ~20%, and would invalidate paid ARM evidence.
  Any revisit lands **before** — never after — an ARM evidence trip.
- **TQ no-QJL 2-bit**: no surface exists (Task 96 accepted stop
  condition); resumes only if a storage decision creates a consumer.
- **f32 raw**: the canonical no-kernel cell on every AM.
- **hamming32**: popcount-bound; NEON 1.10–1.17× accepted below-floor
  result, SVE scoped out by rule, AVX2 skip with measurements
  (POPCNT 11.5–11.8 ns/c ≈ 0.5% of query time).
- **tiled_lut32**: retired (47–48% slower than full_lut at the only
  shipped dimension; cache rationale void at 1536d).

### 9. Named open gaps (not blocking this record)

- **SPIRE pq_fastscan product gap**: the reloption parses but
  `encode_assignment_payload` requires a persisted grouped-PQ model no
  fixture flow provides; no end-to-end SPIRE PQ evidence exists on any
  host (Task 104 finding). Needs an owner decision: wire it or
  document it as a permanent exclusion.
- **HNSW grouped-PQ coverage**: per-candidate traversal scoring; M5
  observed zero batch engagement end-to-end. Candidate for the same
  follow-up discussion.
- Both are recorded in the aggregate matrix §6 and stay visible until
  an operator decision lands.

## Consequences

- New quant work has one template, one evidence bar, one coverage gate,
  and a ratified menu of correctness contracts — review effort goes to
  the family's algebra, not to re-litigating infrastructure.
- The counter surface (not intuition) is the arbiter of where kernel
  wins matter: scoring-share saturation with flat end-to-end is an
  expected, documented outcome class (graph-AM small-frontier cells),
  not a failure.
- The per-ISA columns are owned: scalar (anchor), AVX2 (Task 103),
  Apple-NEON (Task 104), Graviton-4 SVE2 + NEON-capped (Task 99 trip).
  Any new ISA tier (e.g. AVX-512) must ship as a full column with the
  same gates, not as ad hoc cells.

## References

- ADR-076 (pattern), ADR-071/072 (QuantCodec)
- `reviews/task-99/001-aggregate-matrix/artifacts/cross-am-quant-isa-matrix.md`
- `reviews/task-99/000-pre-closeout-architecture-review/feedback/` (F1–F9)
- `reviews/task-99/002-profile-suiteconfig/artifacts/t99-profile-design.md`
- Per-task closeouts: task-87/023, task-92/017, task-93/007, task-94/028,
  task-95/003, task-96/001, task-97/026, task-98/003, task-101/004,
  task-102/001–002, task-103/001–003, task-104/008
