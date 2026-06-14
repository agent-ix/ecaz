---
id: ADR-077
title: "Block Kernel Completeness Closing Record"
status: ACCEPTED
impact: Closes the Task 87/91/92/93-98/101-104 kernel-completeness initiative; sets the operating convention for all future quantized scoring work, the per-family anchor-regime menu, the per-AM enablement policy, and the coverage gate for new quants.
date: 2026-06-11
---
# ADR-077: Block Kernel Completeness Closing Record

Acceptance provenance: ACCEPTED 2026-06-12 by operator decision (the
operator's explicit Task-99-completion directive), with the §4 IVF
default decision and the §6 dispatch-preference decision data-filled
from the three-lane profile (`reviews/task-99/` packets 003/008/009).
No outside-reviewer feedback existed on packets 001–009 at acceptance
time; the outside reviewer is invited to review post-hoc and may reopen
any section — reopened findings route through the Task 99 bucket.

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
  **decision input now measured** (Task 99 profile, three lanes):
  batch-on wins IVF turboquant overwhelmingly (local −66/−69%, G4
  −44%, byte-equal recall), wins IVF pq_fastscan despite the
  suffix-max trade (−5 to −10% on every lane), is neutral on IVF
  rabitq1 (rerank-dominated), and is mildly negative only on the IVF
  QJL @1024 small-nprobe cells (~+8% local, ~0 production lanes; no
  batch counters emit there, indicating the kernel path is not the
  delta). **Decision: flip `ec_ivf.scratch_soa_batch_decode` default
  to on** (implementation is a confirmed follow-up slice with the GUC
  default + docs + a local A/B re-check; the off switch remains for
  diagnostics and for the QJL small-fixture niche).

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
- **Measured outcome (2026-06-12, packets 008/007): SVE2 loses to NEON
  on Graviton 4 at every family where it dispatches** — lut32 2.0–3.3×
  slower, grouped-pq (gather shape) 1.1–1.35×, qjl32 block path ~6×
  vs the pure-NEON cascade; end-to-end −27% to −45% p50 recoverable on
  every TQ/lut32 cell by NEON dispatch, worst regression +0.6%.
  Control cells (rabitq/int8/hamming, NEON-routed in both runs)
  measured identical. **Decision: `select_highest_isa` SHALL prefer
  Neon over Sve/Sve2 on aarch64.** Existing SVE2 kernels stay in-tree
  behind the dispatcher; per-family SVE2 re-entry requires beating the
  NEON cell on the production target. (Implementation is a small
  dispatcher change + G4 re-validation of the changed preference — a
  confirmed follow-up slice; until it lands, G4 operators can set
  `ecaz.isa_cap=neon` to get the measured-better behavior.)

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
  shipped dimension; cache rationale void at 1536d). The SVE column is
  also `retired`, not "skip by rule" — there is no SIMD kernel to apply a
  flush-width rule to (Task 106 audit correction).
- **RaBitQ bits=8 block kernel**: no block kernel by family contract —
  a full-byte quantization level has no LUT fast-scan shape, so bits=8
  uses the arithmetic batch estimator (recall 0.9820 on M5). bits=2/4
  *do* have a block kernel as of Task 106 (the rabitq32 multi-bit family);
  only bits=8 is the deliberate no-block-kernel cell. RaBitQ on HNSW and
  DiskANN is the bits-1 sidecar prefilter by construction — multi-bit is
  not exposed there because a multi-bit *coarse prefilter* is not a useful
  surface.

### 9. Named open gaps — all four closed/decided by Task 106 (2026-06-12; HNSW grouped-PQ measured-skip 2026-06-14)

Sharpened during the Task 105 full-scale sweep, then closed or decided by
the Task 106 targeted pass. The Task 106 audit additionally found that
**multi-bit RaBitQ (IVF bits=2/4) was a weak deferral**, not a true gap —
the bits=1 kernel shipped while 2/4 fell to per-candidate scoring with no
counter attribution and no scoring-share measurement — and closed it too.
The HNSW grouped-PQ cell closed last, as a measured skip, once the AWS
flush-width histogram landed (2026-06-14). The unified-driver coverage claim
now holds with the following resolutions:

1. **SPIRE × RaBitQ — CLOSED in code; e2e effect below noise.** Migrated
   onto the unified candidate-batch driver via a shared
   `score_rabitq_payload_slab` helper: bits=1 engages the rabitq32 block
   kernel (counters + width cascade); bits=2/4/8 use the multi-bit
   arithmetic estimator; the GUC-off path records scalar counter rows. The
   path is now correct and counter-capable. **Honest caveat (M5 index-level
   bench, real 10k):** `ec_spire.candidate_batch_scoring` on/off still shows
   ~0% e2e delta and zero counters on the default bits=4 lane — SPIRE e2e is
   routing/rerank-dominated and assignment scoring is negligible, so the
   "inert toggle" is largely inherent to SPIRE, not fully resolved by the
   migration. SPIRE storage is bits=4 by default, so block-kernel counters
   are bits=1-by-contract here.
2. **HNSW × grouped-PQ — CLOSED (measured skip, 2026-06-14).** A
   measure-first width probe (`record_grouped_pq_traversal_flush_width`,
   commit `06020c8c0`) recorded the flush-width histogram at the grouped-PQ
   traversal boundary on both AWS lanes (10k/50k/100k/1m × ef 40/80/120).
   Per-lane buckets: `width_lt8≈2496`, `width_8_15≈45516`, `width_16_31≈181377`,
   `width_ge32=0` — widths are **16–31 dominant (~79%) and never reach a full
   32-block.** Decision: **do not add a grouped-PQ traversal block kernel on
   HNSW.** The skip is *not* because the widths are too small (16–31 is
   kernel-viable — the same grouped-PQ block kernel wins −3.8/−10.4% on IVF
   where it is kernel-dominant, matrix §2.6). It is because on HNSW grouped-PQ
   scoring is a small share of query time (graph traversal dominates; §7.2,
   graph AMs are width-bound; M5 observed zero batch engagement), and the
   histogram confirms the widths never reach the full-block regime — so the
   e2e benefit would be negligible. **Caveat:** the block kernel was never
   wired into HNSW traversal, so there is no HNSW grouped-PQ e2e A/B — this is
   a histogram-plus-prior-evidence measured skip (the Task 98 method permits
   this), not a measured e2e regression. The width probe stays in code for
   re-measurement if a future change makes grouped-PQ scoring a larger share.
3. **IVF × TQ-QJL — CLOSED (code).** Root cause found: `StorageFormat::Auto`
   (the default) resolves to TurboQuant at scan time but
   `use_scratch_soa_batch_decode_for_format` rejected `Auto`, leaving
   default 10k×1024d indexes on the per-candidate path with zero batch
   counters. The Task 97 512/4096-row fixtures set `storage_format`
   explicitly, which is why they engaged. The gate now admits `Auto` at
   bits=4 like explicit TurboQuant.
4. **SPIRE pq_fastscan — CLOSED (permanent exclusion, existing behavior).**
   Operator decision: SPIRE will not gain grouped-PQ model persistence. The
   `pq_fastscan` reloption parses and an empty index can be created, but a
   populated build defers/errors and the assignment payload reports
   `deferred_model_metadata` / unscannable via the options snapshot — the
   existing, documented exclusion. (An earlier attempt to reject the
   reloption at parse was reverted: it broke the existing deferred-state
   observability and its pg tests — reviewer 2026-06-13-01 P1.)

**Multi-bit RaBitQ (IVF bits=2/4) — CLOSED (code + M5 evidence).** Added
the multi-bit rabitq32 block-kernel family (scalar anchor + NEON + AVX2;
SVE→NEON) and measured it on M5, which set the routing on evidence rather
than assumption:
- **bits=2 → block kernel** (2.66× win over scalar; no per-candidate SIMD
  kernel exists for bits=2). Counter-attributed.
- **bits=4 → per-candidate arithmetic estimator** (NeonBits4) — the block
  kernel measured 2.8× *slower* on M5 NEON, so it is not used. AVX2's
  hardware gather *may* beat the arithmetic path, but the Task 106 AWS run did
  **not** measure that (bits=4 ran the arithmetic path on AVX2; there was no
  forced-block A/B). Current routing (bits=4 arithmetic) stands; the
  AVX2-gather revisit is an explicit **future-optional**, not a Task 106
  deliverable.
- **bits=8 → arithmetic estimator** (full-byte, no LUT fast-scan shape).
This is the §6 dispatch-preference principle applied per bit width: the
block kernel is preferred only where measured faster than the existing
per-candidate path. **Per-ISA picture complete (Task 106 AWS, 2026-06-14):**
the bits=2 block kernel engages on both `avx2` (AWS Intel) and `neon` (G4)
with truthful counters (kernel_candidates large, scalar_candidates=0);
bits=4/8 run the arithmetic estimator on both ISAs as designed.

**Reasoned boundary recorded (Task 106):** TQ-QJL on DiskANN non-1536 is a
deliberate architectural boundary, not a gap — DiskANN's TurboQuant lane is
the compact no-QJL 4-bit *prefilter* search code (1536-only); non-1536 is
served by RaBitQ and grouped-PQ, and a QJL residual-signs prefilter is not
justified. See the aggregate matrix §4.

Remaining: none for Task 106. The cross-host benches are done (Task 106 AWS,
2026-06-14: AWS Intel `avx2` + G4 `neon` confirmed the per-ISA routing —
bits=2 block, bits=4/8 arithmetic) and the HNSW grouped-PQ flush-width
histogram is captured (gap 9.2, measured skip). Future-optional only: the
bits=4 AVX2-gather revisit.

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
