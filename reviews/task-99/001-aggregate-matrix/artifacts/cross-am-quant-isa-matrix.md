# Cross-(AM × Quant × ISA) Block Kernel Completeness Matrix — Task 99 AC1/AC5

Aggregation of the per-task closeout evidence from Tasks 87, 92, 93–98,
101, 102, 103, and 104. No new measurement was run for this document;
every number is cited from its source packet (each packet records its own
head/backend SHA). Compiled at branch `task-99-closeout`, head
`df2765e32` (post Task-104 merge, `origin/main` = `239f27887`).

Column convention for ISA status: **scalar** is the per-family anchor and
exists by construction for every shipped cell; **avx2** is the Intel
column (closed by Task 103 AC6); **neon** is the Apple-silicon
supported-target column (closed by Task 104, `reviews/task-104/008-…`);
**sve2** is the Graviton 4 production column (pending — the single AWS
trip this task owns). NEON numbers below are M5 unless marked otherwise;
G4 NEON/SVE2 numbers do not exist yet anywhere in the project.

## 1. Top-level matrix (AM × quant family)

| Quant family ↓ / AM → | HNSW | IVF | SPIRE | DiskANN |
| --- | --- | --- | --- | --- |
| TQ no-QJL 4-bit (lut32) | **real** | **real** | **real** | **real** |
| TQ no-QJL 4-bit, int8_approx mode (int8_approx32) | **real** | structurally absent (HNSW exact-mode GUC) | structurally absent | structurally absent |
| TQ no-QJL 4-bit, tiled_lut mode (tiled_lut32) | **retired** | structurally absent | structurally absent | structurally absent |
| TQ-QJL (qjl32, non-1536 dims) | **real** | **real** | **real** | structurally absent (TQ storage is 1536-only on DiskANN) |
| TQ no-QJL 2-bit | absent — no surface (Task 96 stop) | absent | absent | absent |
| RaBitQ bits=1 (rabitq32) | **real** (bits-1 sidecar lane) | **real** | **real** (unified driver; Task 106) — counters batch-attributed | **real** |
| RaBitQ bits=2 (rabitq32 multi-bit) | structurally absent (no `quant_bits` on ec_hnsw) | **real** (multi-bit block kernel; Task 106) — no per-candidate SIMD kernel exists for bits=2, and the block kernel is a measured win: M5 NEON **2.66×** (30.7→11.5 µs / 32-block, d1024) | not exposed | bits=1 by construction |
| RaBitQ bits=4 | structurally absent | **real** via the per-candidate arithmetic estimator (NeonBits4/Avx2Bits4) — the multi-bit *block* kernel measured **2.8× slower** on M5 NEON (12.9 vs 4.6 µs), so bits=4 is **not** block-routed (evidence-driven; AVX2 gather may revisit). No block-kernel counters | bits=4-by-default | bits=1 by construction |
| RaBitQ bits=8 | structurally absent | batched arithmetic estimator (NeonBits8/Avx2Bits8); **no block kernel** — full-byte level, no LUT fast-scan shape | not exposed | bits=1 by construction |
| grouped-PQ (grouped_pq_block) | registered; traversal-batch decision is measurement-gated (§6a item 2) | **real** | **structurally absent — product gap, permanent exclusion** (see §4) | **real** |
| binary/Hamming sidecar (hamming32) | out of scope (Task 95 scope: DiskANN only) | out of scope | out of scope | **real** (NEON) / **skip** (AVX2, documented) |
| f32 raw | no-kernel cell (canonical) | no-kernel cell | no-kernel cell | no-kernel cell |

## 2. Per-family evidence (shipped cells)

### 2.1 lut32 — TQ no-QJL 4-bit, dim 1536 (flagship lane)

Contract: bit-exact, scalar dim-order accumulation preserved per lane
(ADR-076 strict lane). Anchor established `reviews/task-87/016…`,
kernels `reviews/task-102/`.

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| SPIRE real10k | avx2 | 1,054–1,062 → 235–237 ns/c (**4.5×**; 5.6× vs unbatched anchor) | nprobe=32 p50 17.3 → 8.54 ms (**−50.6%**) | byte-equal (1.0000) | task-102/001 manifest |
| HNSW real10k full_lut | avx2 | 509–530 ns/live-candidate, multi-lane (**2.8×** vs same-head scalar) | ef=80 p50 16.5 → 4.65 ms (**−71.8%**); kernel-on now beats kernel-off (4.65 vs 5.22 ms) | byte-equal (0.6240/0.9321) | task-102/001 manifest |
| DiskANN 2k TQ | avx2 | 265–283 ns/c (octet padding) | list=64 −26.3%, list=128 −29.8% | byte-equal (0.7516/0.9726) | task-102/002 manifest |
| IVF real10k | neon (M5) | 221.3–221.9 vs 880.9–912.9 ns/c (**~4.1×**) | nprobe 8/16/32: −53.5/−59.3/−62.7% | byte-equal | task-104/008 matrix |
| HNSW real10k full_lut | neon (M5) | 494.9 vs 903.3 ns/c (**1.83×** — Task 102 repack did NOT regress on real NEON) | ef 32/80/200: −8.2/−5.1/−4.8% | byte-equal | task-104/008 matrix |
| SPIRE real10k | neon (M5) | 226.8 vs 818.8 ns/c (**3.61×**) | nprobe 8/16/32: −9.6/−12.4/−11.9% | byte-equal | task-104/008 matrix |
| DiskANN real10k | neon (M5) | 298.7 vs 891.1 ns/c (**2.98×**) | list 64/128: −35.2/−38.3% | byte-equal | task-104/008 matrix |
| IVF/SPIRE real10k–100k | scalar (pre-kernel batching, Task 87) | batch-amortization only | p50 −13.5% to −18.1% (10k), −14.8/−15.3/−14.9% (50k/100k) | byte-equal | task-87/021/022/024 |
| any | sve2 | **pending G4** | pending | pending | this task's trip |

Note: no IVF-specific Intel lut32 cell was measured post-Task-102 (the
102 evidence covers SPIRE/HNSW/DiskANN; the kernel is the same dispatch
entry). The Task 99 profile fills the IVF×lut32×avx2 cell explicitly.

### 2.2 int8_approx32 — HNSW exact-score mode

Contract: integer-exact (i32 accumulation, order-independent) → strict
`to_bits()` equality across all ISA backends — the strongest contract in
the project. Established `reviews/task-98/`, AVX2 `reviews/task-103/002…`.

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| HNSW real10k | avx2 | **88.6 ns/c vs 918.7–923.0 anchor (10.4×**, gate ≥2×) | ef=80 4.52 → 3.63 ms (−19.7%); fastest measured HNSW exact mode (beats full_lut 4.52/6.76) | byte-equal (0.6230/0.9319) | task-103/002 manifest |
| HNSW real10k | neon (M5) | 99–105 ns/c, kernel-dominant (fastest HNSW exact mode on M5) | ef 32/80/200: −10.4/−13.8/−12.5% | byte-equal | task-104/008 matrix |
| HNSW (NEON, pre-M5) | neon | ~300 ns/c full coverage | 100k within noise | byte-equal all 6 mode×corpus cells | task-98/003 closeout |
| any | sve2 | **skip by rule** (Task 98 AC4: ≥32-wide flushes ≤0.08% of HNSW frontier distribution) | — | — | task-98/003 closeout |

### 2.3 tiled_lut32 — retired

- Intel A/B (task-103/001): tiled_lut scalar 2,994–3,001 ns/c vs full_lut
  AVX2 492–546 ns/c; end-to-end 47–48% slower than full_lut. Cache
  rationale void at dim 1536 (LUT is L1-resident). **Decision: retired/
  deprioritized; no SIMD built for a losing lane.**
- M5 confirmation (task-104/008): NEON path is a scalar-delegating stub
  at 1,339.6 ns/c vs full_lut's 495 ns/c kernel — retired marker
  executed as confirmation using the new runnable `retired`
  kernel_status (`d1235077c`).

### 2.4 qjl32 — TQ-QJL (gamma + residual signs), non-1536 dims

Contract: ADR-076 tolerance lane — forced-scalar bit-exact anchor +
4-ULP/1e-6 per-slice dispatch pair; recall preservation binding. The
Task 97 packet 015 diagnostic (5,920 ULP from cancellation under
reordering) is the cautionary citation for why this lane exists.

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| SPIRE 512/4096 @1024d | avx2 | direct **2.48–2.97×** (target band 1.8×+) | ~neutral e2e (+1.00–1.04×) | byte-equal 14/14 pairs | task-97/026 |
| IVF 512/4096 @1024d | avx2 | counter-attributed | e2e **+1.14× to +1.71×** | byte-equal | task-97/026 |
| HNSW 512/4096 @1024d | avx2 | counter-attributed | +1.04–1.10× (one cell 0.96×, noted) | byte-equal | task-97/026 |
| IVF 10k @1024d | neon (M5, octet round `5c44d9f45`) | 167.9–168.6 vs 585.1–602.3 ns/c (**~3.5×**; was 0.83× pre-Task-104-fix) | nprobe 8/16: −47.8/−55.1% | byte-equal | task-104/008 |
| HNSW 10k @1024d | neon (M5) | 168.3–168.9 vs 581.5–584.8 ns/c (**~3.5×**) | ef 32/80: −17.4/−13.5% | byte-equal | task-104/008 |
| SPIRE 10k @1024d | neon (M5) | 168.6–169.2 vs 595.0–602.2 ns/c (**~3.5×**) | nprobe 8/16: −5.6/−9.4% | byte-equal | task-104/008 |
| any | sve2 | **pending G4** (runbook `reviews/task-97/022…`; must capture measured vector length, expected `sve2-128`) | pending | pending | this task's trip |

Task 97 status note: **in review, approval-gated on the G4 evidence** —
the only Phase III family task not yet `complete`. Its closure rides this
task's AWS trip (same single-trip economics).

### 2.5 rabitq32 — RaBitQ bits=1

Contract: bit-equal with the production same-order path by construction
+ 1e-5 relative envelope vs the forced-scalar anchor (measured 22 ULP /
1.55e-6 at dim 1536 under FMA reordering) + recall-binding. Established
`reviews/task-93/002…/003…`.

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| IVF real10k/100k | neon | 223/191/126 ns/c vs 793/515/364 scalar (**2.69–3.55×**) | real100k p50 3.57→3.82 ms (~7%) | byte-equal | task-93/002/003/006 |
| HNSW real10k–100k | neon | 137–230 ns/c, partial-width (avg ~22) | — | byte-equal | task-93/004/006 |
| DiskANN real10k–100k | neon | 236–285 ns/c, partial-width (avg ~10) | — | byte-equal | task-93/004/006 |
| DiskANN 2k bits=1 | avx2 | **80.4–81.1 ns/c**, scalar_candidates=0, width histogram exercises full partial range | e2e within noise both sweeps | byte-equal (0.5984/0.9541) | task-103/003 manifest |
| IVF / HNSW / DiskANN 10k | neon (M5) | 63.9–65.6 ns/c, kernel-dominant | IVF ~0% (rerank-dominated), HNSW −5.7 to −10.3% | byte-equal | task-104/008 |
| SPIRE | any | **e2e only — counters not batch-attributed on this surface** (M5 finding) | measured | — | task-104/008 §1 |
| any | sve | kernel exists (`src/quant/rabitq32/sve.rs`), unmeasured; SVE hosts currently route NEON with truthful `isa=neon` attribution. Decision deferred to this task with the datum: IVF is the one surface with real 32-block coverage (~99%) where SVE could pay beyond NEON | pending G4 | pending | task-93/007 closeout |

### 2.6 grouped_pq_block — grouped-PQ / PqFastScan

Contract: bit-exact vs scalar f32-LUT reference (survived the F8
shuffle-repack; `grouped_pq_block32_matches_scalar_reference_bits_across_shapes`).

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| IVF 10k/25k/100k | avx2 (F8 repack + 101 cascade) | kernel_candidates up to 50M/sweep, scalar_candidates=0 | batch-on beats batch-off at all 6 cells: **−3.8% to −10.4%** | byte-equal all cells | task-94/028 (AC5 rerun, release backend) |
| DiskANN 50k/100k forced grouped-pq | avx2 | kernel-attributed, coverage ~3% → **≥80–100%** post-Task-101 octet/partial | 3/4 cells positive, 1 within stddev | byte-equal | task-94/025/027/028, task-101/004 |
| IVF 10k | neon (M5) | **30.4–30.9 ns/c** (kernel-dominant, 980k candidates) | ±4% e2e (kernel share small) | byte-equal | task-104/008 |
| DiskANN 10k default | neon (M5) | routes through binary sidecar prefilter by default; grouped-PQ arm engages via `prefilter_kind=grouped_pq` only (recorded) | −2.1/−7.8% (sidecar-routed) | byte-equal | task-104/008 |
| any | sve2 | gather-shape kernel landed; repack decision deferred — if unrepacked at G4 time, annotate G4 rows as measuring the gather shape | pending G4 | pending | task-94 task file (reopened scope) |

Pruning interaction (decoupling input): the IVF batch path trades away
suffix-max cutoff pruning (task-94/024 F1); `ec_ivf.scratch_soa_batch_decode`
remains default-off pending this task's profile (F4 enablement-policy
paragraph in ADR-077).

### 2.7 hamming32 — binary/Hamming sidecar

Contract: integer-exact (XOR + popcount).

| Cell | ISA | scoring-share | end-to-end | recall | source |
| --- | --- | --- | --- | --- | --- |
| DiskANN real10k/100k | neon | **1.17× / 1.10×** (hardware-popcount-bound; below 1.5× floor, accepted stop-condition) | p50 −21% (10k, allocation elimination) / −3% (100k) | byte-equal (0.9938/0.9719) | task-95/002 manifest |
| DiskANN 10k | neon (M5) | 7.1 ns/c | — | byte-equal | task-104/008 |
| DiskANN | avx2 | **skip, documented**: scalar POPCNT at 11.5–11.8 ns/c ≈ 0.5% of query time; even 2× → ≤0.3% e2e | batch on/off within noise | — | task-103/001 manifest |
| any | sve | **scoped out by rule** — same-algebra `cnt` structurally bounded by the NEON 1.10–1.17× result; consuming surface batches ~10 wide | — | — | task-95/003 closeout |

### 2.8 Pre-kernel infrastructure rows

- **Task 87** shipped the batch plumbing + scalar lut32 and the original
  per-AM counter surface; its measured wins (−13% to −25% p50/p95/p99 on
  IVF/SPIRE TQ cells, recall byte-equal) predate the SIMD kernels and are
  the batching-amortization baseline every kernel number above stacks on.
- **Task 92** shipped ADR-076 (ACCEPTED), the `(surface × quant × isa)`
  counter matrix, the `Isa` dispatcher, module layout, `kernel_status`
  markers, and the dev doc — dispatch overhead measured ≤1% of kernel time.
- **Task 101** unified all seven families behind one width-cascade driver
  (32 → octet → partial → scalar), backported prevalidation everywhere,
  added `TurboQuantTiledLut`/`TurboQuantInt8` counter kinds, and took
  DiskANN grouped-PQ SIMD coverage from ~3% to full attribution; release
  rerun improved all six IVF cells (−3.8% to −10.4%).

## 3. f32 raw — canonical no-kernel cell (AC item 3)

The unquantized f32 lane (heap_f32 rerank and raw scoring) has no block
kernel on any AM or ISA, and none is planned: scoring is a plain dot
product the compiler auto-vectorizes; there is no LUT/decode stage for a
block kernel to amortize. Recorded `structurally_absent` on every AM, per
the Task 92 marker convention and the Task 104 matrix's identical entry.

## 4. Structural exclusions (AC2/AC5 — every absent cell, with source)

| Cell | marker | source evidence |
| --- | --- | --- |
| TQ no-QJL 2-bit, all AMs | `structurally_absent` — no AM exposes a 2-bit no-QJL surface; `qjl_enabled()` makes bits=2 QJL-by-construction; SQL rejects bits≠4 | task-96/001 surface inventory + accepted stop condition (2026-06-09) |
| HNSW exact-score modes on IVF/SPIRE/DiskANN | `structurally_absent` — `ec_hnsw.turboquant_exact_score_mode` is an HNSW GUC; other AMs have no exact-mode stage | task-98 task file scope |
| TQ-QJL on DiskANN (non-1536) | **reasoned boundary** (Task 106) — DiskANN's TurboQuant lane is the compact no-QJL 4-bit *prefilter* search code (`VAMANA_SEARCH_CODEC_TURBOQUANT`), which only exists at dim=1536. Non-1536 TQ requires QJL residual signs — a different, larger code format not justified for a coarse prefilter, and DiskANN already serves non-1536 via RaBitQ (bits-1 sidecar) and grouped-PQ. Not built; non-1536 users choose RaBitQ/grouped-PQ. | task-104/008; Task 106 slice 9 |
| RaBitQ bits=2/4 kernels (IVF) | **real** (Task 106) — multi-bit rabitq32 block kernel (scalar anchor + NEON + AVX2; SVE→NEON), routed through the unified driver with truthful counters. NEON M5-validated; AVX2 bench-pending. Was a weak deferral (bits=1 shipped, 2/4 punted) | Task 106 slices 5–8 |
| RaBitQ bits=8 kernels | no block kernel by family contract — full-byte level has no LUT fast-scan shape; uses the arithmetic batch estimator (recall 0.9820 on M5) | task-93/002 scope; Task 106 |
| RaBitQ bits=2/4/8 on HNSW/DiskANN | `structurally_absent` — HNSW/DiskANN expose no `quant_bits`; rabitq there is the bits-1 sidecar lane by construction (a multi-bit *sidecar prefilter* is not a useful surface) | task-104/008 markers |
| grouped-PQ on HNSW | **measurement-gated** (Task 106 slice 2) — traversal scores per-candidate; the rabitq traversal batch boundary (`flush_rabitq_search_code_batch`) exists to mirror if the flush-width histogram (Task 98 Phase A) justifies it. ADR-077 §7.2 (graph AMs are width-bound) + M5 zero-engagement point toward a measured skip; the histogram on the bench hosts decides | task-94; task-104/008 §1; Task 106 |
| **grouped-PQ on SPIRE** | `structurally_absent` — **permanent exclusion** (Task 106 slice 4): SPIRE has no grouped-PQ model persistence, so the index cannot build. The `pq_fastscan` reloption is now rejected at parse with a clear message (operator decision: SPIRE will not gain model persistence) | task-104/008; Task 106 slice 4 |
| Hamming on IVF/HNSW/SPIRE | out of scope — Task 95 scope is the DiskANN binary-sidecar prefilter, the only ≥batch-width binary surface; IVF/HNSW/SPIRE have no persisted binary-sidecar surface (true gap) | task-95 task file |
| tiled_lut32, all ISAs | `retired` | task-103/001; task-104/008 confirmation |
| hamming32 AVX2 | `skip` (documented with measurements) | task-103/001 |
| int8_approx32 SVE | skip by rule (HNSW ≥32-wide flushes ≤0.08%, measured) | task-98/003 |
| tiled_lut32 SVE | `retired` (no SIMD kernel exists to apply the width rule to — tiled_lut32 is a scalar-stub retired lane; the "skip by rule" label was a miscategorization) | task-103/001; task-104/008; Task 106 audit |
| hamming32 SVE | scope-down by rule (popcount-bound) | task-95/003 |
| Quantized-LUT (u8 fast-scan) lut32 variant | **deferred indefinitely** (operator decision 2026-06-10; breaks byte-equal regime, post-102 upside ~20%, would invalidate paid ARM evidence) | task 99 file, "Absorbed deferrals" |
| f32 raw, all AMs | `structurally_absent` (canonical no-kernel cell) | §3 above |

## 5. ISA column status (input to AC4, completed after the trip)

| ISA | status | what remains |
| --- | --- | --- |
| scalar | complete — anchor per family by construction | — |
| avx2 (Intel) | **complete** (Task 103 AC6: five real kernels — lut32, qjl32, grouped_pq, int8_approx32, rabitq32 — plus retired/skip decisions; no `missing_kernel` cells) | AWS-Intel profile lane re-measures the same cells on citable hardware |
| neon (Apple M5) | **complete** (Task 104: every family ≥1.5× floor or documented marker; 40/40 recall pairs byte-equal; no-SVE ladder validated) | supported-target column; never substitutes for G4 |
| neon (Graviton 4) | **complete** (trip 2026-06-12, packet 008 neoncap run via `ecaz.isa_cap=neon`: every kernel family measured, zero sve rows under cap, recall unchanged) | — |
| sve2 (Graviton 4) | **complete** (packet 008: `sve2-128` measured; lut32/grouped-pq/qjl32 SVE2 kernels served default dispatch with truthful attribution) — **and measurably loses to NEON at every family** (2.0–3.3× on lut32; e2e −27/−45% recoverable). Dispatch-preference flip decided in ADR-077 §6; rabitq32 SVE kernel question closed (unjustified) | follow-up slice: dispatcher preference change |

## 6a. Unified-driver coverage gaps (sharpened 2026-06-12, Task 105 sweep)

Originally noted for a smaller targeted pass after the full-scale sweep.
**Task 106 (2026-06-12) closed or decided all four**, plus the multi-bit
RaBitQ weak deferral surfaced by the Task 106 audit. Status:

1. **SPIRE×RaBitQ — CLOSED (code).** Migrated onto the unified
   candidate-batch driver: bits=1 engages the rabitq32 block kernel
   (counters + width cascade); the GUC-off path records scalar counter
   rows. `ec_spire.candidate_batch_scoring` is now a live A/B switch
   instead of inert. (SPIRE assignment storage is bits=4 by default, so
   block-kernel counters are bits=1-by-contract — see §1 RaBitQ note.)
2. **HNSW×grouped-PQ — DECIDED, measurement-gated.** Per the Task 106
   slice text and ADR-077 §7.2 (graph AMs are width-bound: greedy descent
   consumes each expansion's scores before forming the next, so
   partial/octet dispatch — not cross-expansion batching — is the
   coverage answer), the resolution is a flush-width measurement
   (Task 98 Phase A histogram) on the bench hosts. M5 already observed
   zero batch engagement e2e; the histogram confirms batch-or-skip. The
   rabitq traversal batch boundary (`flush_rabitq_search_code_batch`)
   exists to mirror if the width distribution justifies it; a measured
   skip is an acceptable outcome.
3. **IVF×TQ-QJL — CLOSED (code).** Root cause: `StorageFormat::Auto`
   (the default when no `storage_format` reloption is set) resolves to
   TurboQuant at scan time, but `use_scratch_soa_batch_decode_for_format`
   rejected `Auto`, leaving default 10k×1024d indexes on the
   per-candidate path with zero batch counters. The Task 97 512/4096-row
   fixtures set `storage_format` explicitly, which is why they engaged.
   The gate now admits `Auto` at bits=4 exactly like explicit TurboQuant.
4. **SPIRE×pq_fastscan — CLOSED (permanent exclusion).** Operator
   decision: SPIRE will not gain grouped-PQ model persistence. The
   reloption is now rejected at parse with a clear message instead of
   parsing then failing at build. See §4.

**Multi-bit RaBitQ (IVF bits=2/4) — CLOSED (code + M5 evidence).** This
was a weak deferral, not a true gap: the bits=1 rabitq32 kernel shipped
while bits=2/4 fell to per-candidate scoring with no scoring-share
measurement. Task 106 added the multi-bit block-kernel family (scalar +
NEON + AVX2; SVE→NEON) **and measured it on M5**, which decided the
routing on evidence:
- **bits=2 → block kernel.** No per-candidate SIMD kernel exists for
  bits=2 (estimate_ip_batch supports only 1/4/8), so it otherwise falls
  to true scalar. The block kernel is a **2.66× win** (M5 NEON: 30.7→11.5
  µs for a 32-block at d1024). Counter-attributed.
- **bits=4 → per-candidate arithmetic estimator (NeonBits4/Avx2Bits4).**
  The block kernel measured **2.8× slower** on M5 NEON (12.9 vs 4.6 µs):
  its per-dim scalar gather doesn't beat NeonBits4's vectorized nibble
  unpack. So bits=4 is **not** block-routed. The AVX2 hardware gather
  (`permutevar8x32`) may change this on the Intel lane — a per-ISA
  routing decision to confirm there.
- **bits=8 → arithmetic estimator** (full-byte, no LUT fast-scan shape).

The block kernel remains built and tested for all ISAs so the Intel/G4
benches can complete the per-ISA routing picture. See §1 and §2.5.

## 6. Open items this matrix feeds forward

1. **G4 lane** (profile + per-family runbooks 94/027-shape, 97/022) — the
   only missing ISA column; closes Task 97 and the rabitq32/grouped-pq SVE
   decisions.
2. **AWS Intel lane** — same profile config, citable Intel hardware,
   instance types + pricing recorded (procurement question).
3. **SPIRE PqFastScan product gap** — needs an owner decision: either a
   follow-up task to wire `encode_assignment_payload` to a persisted
   model, or a documented permanent exclusion in ADR-077.
4. **HNSW grouped-PQ coverage gap** (M5 finding) — Task 94/101 sub-width
   backport lane; candidate for the same follow-up discussion.
5. **IVF×lut32×avx2 direct cell** — covered by the profile (§2.1 note).
6. **SPIRE×rabitq32 counter attribution** — covered by the profile or
   documented as e2e-only.
7. **Enablement-policy paragraph** (F4) + **anchor-regime menu** (F5) +
   **counter-key disambiguation note** (F2) — ADR-077 content, drafted in
   the next packet.
