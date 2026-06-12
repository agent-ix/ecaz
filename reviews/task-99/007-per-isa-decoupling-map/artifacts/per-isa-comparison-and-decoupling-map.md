# Task 99 Phase 3 — per-ISA comparison + scoring-share vs end-to-end decoupling map

Status: **final** — all five columns measured. Local-Intel (packet 003)
and Apple-M5 (task-104/008) plus the trip lanes: Graviton 4 default +
NEON-capped (packet 008) and AWS Intel (packet 009).

## 1. Per-ISA comparison (AC 4) — kernel ns/candidate by family

| family | scalar anchor | avx2 (local Intel, 003) | neon (Apple M5, 104) | G4 default dispatch (008) | G4 neon-capped (008) | avx2 (AWS Intel, 009) |
| --- | --- | --- | --- | --- | --- | --- |
| lut32 (IVF/SPIRE blocks) | 1,054–1,062 (task-102) | 235.6–255.1 | 221.3–226.8 | sve2: 1,204/1,210 | **589/597** | 170/180 |
| lut32 (HNSW multi-lane) | ~1,309 unbatched | 477.3–479.5 | 494.9 | sve2: 3,897 | **1,184** | 374 |
| lut32 (DiskANN) | 891.1 (M5 anchor) | 277.9–278.2 | 298.7 | sve2: 1,504 | **710** | 213 |
| int8_approx32 (HNSW) | 918.7–923.0 (task-103) | 86.8–87.4 | 99–105 | neon: 129 (SVE skip by rule) | 129 (control) | 96 |
| rabitq32 | 364–793 (task-93, per nprobe) | 68.8–89.8 | 63.9–65.6 | neon: 115–182 (truthful routing) | identical (control) | 71–92 |
| grouped_pq (IVF) | ~110 scalar rate (task-94 F8 datum) | 160.4–161.6¹ | 30.4–30.9¹ | sve2: 144 (gather shape) | **130** | 132 |
| grouped_pq (DiskANN) | 〃 | 143.6–156.9 | sidecar-routed | sve2: 160 | **119** | 138 |
| qjl32 @1024d | 581–602 (M5 anchor) | 256.0–263.5 | 167.9–169.2 | sve2 blocks ~3,000 + neon octets | **429–483** | 215–235 |
| hamming32 | 11.5–11.8 POPCNT (task-103) | skip (decision) | 7.1 | neon: 12 | 12 (control) | scalar POPCNT (skip upheld) |
| tiled_lut32 | 1,339.6–3,001 (retired) | retired | retired | scalar (retired) | — | scalar (retired) |

¹ grouped-PQ rates are per *posting candidate* on different decode
shapes per host lane — compare within a column, not across AMs.

**Verdict (the §3-slot answer for ADR-077 §6): SVE2 loses to NEON on
Graviton 4 at every family where it dispatches** — 2.0–3.3× slower on
lut32, 1.1–1.35× on grouped-pq (gather shape), and the qjl32 SVE2
block path ~6× slower than the pure-NEON cascade. End-to-end, default
(SVE2-preferred) dispatch costs **27–45% p50 on every TQ/lut32 cell**
(IVF TQ 41.0 vs 22.9 ms at nprobe=16), 17–21% on SPIRE QJL, ~5% on IVF
pqfs; worst neon-capped regression anywhere is +0.6% (noise). The
already-NEON families measure identical capped vs uncapped — internal
controls validating the comparison. Recommendation: flip
`select_highest_isa` to prefer NEON over Sve/Sve2 on aarch64 (SVE2
re-entry per family only by future measurement).

Cross-ISA observation: AVX2 (256-bit) beats NEON (128-bit) by ~1.3–2×
on most families on production hosts, and M5-NEON ≈ AWS-Intel-AVX2 on
several (M5 silicon is strong per clock); G4-NEON trails both. The
grouped-pq family is the exception where G4-NEON ≈ AVX2.

## 2. Decoupling map (AC 3 / scope item 5) — where kernel wins reach end-to-end

Classification from packet 003 (batch-on vs batch-off p50, 100k real
fixtures; QJL cells 10k @1024d). Kernel saturation = `scalar_candidates=0`
on every kernel row (holds at every kernel-on cell).

### Class A — scoring-dominated: kernel wins flow through

| cell | e2e delta (per sweep) | dominating stage when off |
| --- | --- | --- |
| IVF turboquant | **−66.1% / −69.1%** | per-candidate LUT scoring |
| SPIRE turboquant | **−47.9% / −62.3%** | leaf scoring |
| DiskANN turboquant | −16.7% / −30.6% | prefilter scoring |
| HNSW qjl @1024 | −22.0% / −21.6% | exact-stage scoring |
| HNSW int8_approx | −20.1% / −13.0% | 〃 |
| HNSW full_lut | −8.8% / −12.6% | 〃 |

### Class B — pruning-trade cells: kernel wins net against forfeited cutoffs

| cell | e2e delta | verdict |
| --- | --- | --- |
| IVF pq_fastscan | −5.4% / −10.4% | suffix-max forfeit still nets a win at 100k (consistent with task-101/004) |
| IVF qjl @1024 (nprobe 8/16) | **+8.3% / +3.0%** | trade nets a LOSS at small nprobe/small fixture — first measured negative-net batch cell; input to the IVF GUC default decision (ADR-077 §4) |

### Class C — other-stage-dominated: kernel saturated, e2e flat

| cell | e2e delta | dominating stage |
| --- | --- | --- |
| IVF rabitq1 | +3.6% / −3.2% | heap_f32 rerank (width 50) |
| SPIRE rabitq | −1.8% / −0.2% | pipeline; also no batch counter attribution on this surface (finding 2, packet 003) |
| SPIRE qjl @1024 | −1.7% / −1.1% | SPIRE routing/pipeline |
| HNSW rabitq | +0.7% / +3.9% | graph traversal (task-93 small-frontier finding reproduced) |

### Class D — structurally no-axis cells

| cell | note |
| --- | --- |
| DiskANN grouped-pq | prefilter arm ungated — batch always-on; on/off deltas ~0 by construction (packet 003 finding 1) |
| HNSW exact mode | no kernel; on/off within noise (+4.5%/+3.5%) — clean baseline |
| IVF rabitq4 | no-kernel storage lane; p50 20.4/59.7 ms vs rabitq1 8.05/18.2 at equal nprobe (bits-tradeoff datum for ADR-025) |
| DiskANN binary | hamming AVX2-skip stands; see packet 003 recheck for the flagged cell |

### Cross-host note (M5 vs local Intel)

The class assignment is mostly host-stable (IVF/DiskANN TQ class A on
both; IVF rabitq1 class C on both), with one host-sensitive case:
SPIRE TQ is strongly class A locally (−48/−62% at 100k) but mild on M5
(−9.6/−12.4% at 10k) — fixture-scale-dependent leaf-scoring share, not
an ISA effect. The G4 lane measures the same 100k shape as local.

## 3. What the trip filled in (2026-06-12)

1. G4 SVE2 column measured; vector length **sve2-128** confirmed
   (`/proc/sys/abi/sve_default_vector_length` = 16 bytes + the
   `runtime_sve_vector_lanes_for_test` assertions in the day-one set).
2. G4 NEON-capped column → **NEON wins every family** (verdict above);
   `select_highest_isa` preference change recommended.
3. AWS-Intel column measured (packet 009); price/performance inputs in
   the lane manifests (m8g.2xlarge ~$0.346/hr vs m7i.2xlarge
   ~$0.493/hr status-line rates; per-cell p50s in results.jsonl —
   Intel wins most absolute p50s even before normalizing, and by more
   under today's G4 dispatch).
4. rabitq32 SVE decision closed by the same data: the truthful NEON
   routing is not just acceptable but optimal — building a rabitq32
   SVE2 kernel is measurably unjustified (SVE2 loses at equal width on
   every family that has one).
5. Task 94 gather-shape SVE2 grouped-PQ cells measured + annotated
   (and the NEON repack question answered: G4-NEON already beats
   G4-SVE2-gather); Task 97 G4 cells collected (packet 008
   `task97-run/`, `isa=sve2` rows + runbook gates).

## 4. Decoupling-map deltas from the trip

The class structure holds on both production lanes. Notable cross-lane
rows: IVF TQ stays class A everywhere (G4 −44% e2e under proper NEON
dispatch); DiskANN TQ class A on G4 (−27/−30% capped); the IVF QJL
@1024 class-B negative-net cell stays ~0/slightly negative on both
production lanes (and emits no batch counters on any host — recorded
as an attribution gap with SPIRE-rabitq).
