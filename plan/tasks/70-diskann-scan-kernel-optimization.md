# Task 70: DiskANN Scan Kernel / Per-List Scoring Optimization

Status: proposed
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (load-bearing for closing the cross-engine query-latency gap)

## Why

Task 32 packet 001's cross-engine refresh on M5 real10K measured:

| engine | recall@10 | mean q-time | p50 | p95 | p99 | index size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` (list=64) | 0.9965 | **2.14 ms** | 2.11 | 2.38 | 2.67 | 4.94 MB |
| `pgvectorscale` (list=64) | 0.9960 | **0.60 ms** | 0.59 | 0.73 | 0.88 | 5.14 MB |

At matched recall (within 0.05 pp) and **near-identical index size**
(ec_diskann is actually smaller), `ec_diskann` is 3.5× slower per
query. The gap cannot be explained by storage format, codec
density, or graph layout differences — pgvectorscale's index is
4 % *larger*. The remaining axes are:

- per-list scan loop quality (branch prediction, prefetch, memory
  layout walk)
- scoring kernel (NEON dispatch, FMA throughput, register
  pressure)
- candidate management (heap maintenance, dedup, bounded top-k)
- rerank fetch (heap TID materialisation, detoast cost)
- per-rescan setup (scratch allocation, frontier reset)

Task 29c / Task 65 closed the *build* algorithmic surface. The
*scan* surface has not had an equivalent profiling pass. Task 32
Phase 2/3 (per-scan graph read cache, rerank staging, scratch
reuse, Apple Silicon kernel check) was deferred when Task 32 closed
after Phase 1.

This task picks up Task 32's deferred Phase 2/3 with a sharper
focus: characterise the scan latency split first, then land
targeted slices that close the 3.5× pgvectorscale gap.

## Non-Goals

- Do not change DiskANN on-disk format. Scan-side wins should live
  inside the existing `ec_diskann` page format and graph layout.
- Do not change recall behaviour. Any slice that touches scoring
  must preserve recall@10 within 0.5 pp of the cited fixture's
  reference number (real10K @ L=64: `0.9965`).
- Do not start AWS / Graviton work in this task. M5 is the local
  optimization host (mirror Task 32 baseline rules). Cloud
  confirmation is a separate follow-on (Task 59).
- Do not touch build-side performance — that's Task 65b's scope.
- Do not pursue scoring kernel work that overlaps Task 21 (broad
  SIMD modernisation) without coordinating; if a kernel change
  belongs in the cross-AM SIMD dispatch surface, file it under
  Task 21 instead.

## Phase 1 — Scan-Path Characterization (gating)

Mirror Task 32 Phase 2 with the actual measurement that was
deferred. Land one packet:

- M5 release-mode `ec_diskann` scan at L=64 and L=200, real10K
  fixture.
- Split per-query wall time across:
  - binary sidecar prefilter + popcount,
  - persisted graph page reads + tuple decoding,
  - frontier maintenance (heap push/pop, visited bitset),
  - exact heap rerank fetch + detoast,
  - result materialisation + per-rescan setup.
- Compare to pgvectorscale's matched-L profile (same fixture,
  same matched-recall point).
- Capture `EXPLAIN (ANALYZE, BUFFERS)` for representative low-L
  and high-L queries on both engines.
- Profile via `samply`, `cargo flamegraph`, or `dhat` — whatever
  shows the per-phase wall-time split clearly.
- Rank Phase 2 P0 slices in priority order, each with measured
  share and estimated cap (best-case speedup).

Phase 1 closes when the measurement packet has reviewer-approved
findings and a ranked P0 list. Reuse the Task 68 phase-disjoint
notice pattern if instrumentation requires it: any new in-process
timing should produce disjoint phase totals or explicitly document
nesting.

## Phase 2 — P0 Slices

P0 slices land one at a time, each with:

- Code packet with source diff + Phase-1 backreference.
- Measurement packet repeating the Phase-1 split with the slice
  applied.
- Per-slice cap: skip if projected speedup is below ~5 % of total
  scan wall time at L=64, unless it's a prerequisite for another
  slice.

Candidate slices (only those Phase 1 ranks P0 are landed):

1. **Per-scan graph read cache** — if Phase 1 shows repeated page
   or tuple decoding in one scan, cache decoded graph tuples for
   the scan lifetime. Carried over from Task 32 Phase 3.
2. **Rerank staging** — if exact heap rerank dominates, add a
   measured intermediate ranking stage only when it preserves the
   recall floor. Carried over from Task 32 Phase 3.
3. **Frontier / result scratch reuse** — reuse scan-local buffers
   across rescans where allocation shows up in profiles. Carried
   over from Task 32 Phase 3.
4. **NEON scoring kernel dispatch audit** — verify whether
   `ec_diskann` scoring reaches the best available arm64 backend
   (NEON / SVE / SVE2). If a backend gap is found, route the broad
   SIMD work to Task 21; only narrow per-call dispatch fixes land
   here.
5. **Binary sidecar prefilter tuning** — if popcount + prefilter
   shows as the dominant share, examine the prefilter density,
   threshold tuning, and codebook layout.
6. **Candidate top-k / dedup overhead** — if heap maintenance is
   measurable, evaluate min-heap shape, bounded-K early-stop, or
   bitset-based dedup vs hash-based.

Any slice outside the above list requires Phase-1 evidence and a
short addendum to this task file.

## Exit Criteria

- Phase 1 characterization packet landed with reviewer-approved
  ranking.
- All Phase 1 P0 slices either landed with a measured win on the
  same fixture, or explicitly shelved with a recorded reason.
- Final measurement packet repeating the Phase-1 split, showing the
  scan-time delta vs baseline at L=64 and L=200.
- Recall floor preserved: real10K @ L=64 recall@10 within 0.5 pp
  of `0.9965`; real10K @ L=200 within 0.5 pp of `0.9970`.
- Cross-engine row in `docs/benchmarks.md` updated with the new
  ec_diskann measurement and the residual gap (or closure) vs
  pgvectorscale.
- No new `unsafe { ... }` blocks introduced unless within a
  `target_feature` kernel body with paired `# Safety` doc per
  memory `feedback_dont_defer_safety_fixes` and
  `feedback_anti_pattern_b_unbounded_lifetime`.
- `cargo clippy --all-targets --no-default-features --features pg18
  -- -D warnings` clean.

## Coordination

- **Task 32 is closed**; this task picks up its deferred Phase 2/3
  work with a sharper scope.
- **Task 65b (parallel graph construction) is independent** — that
  attacks the build-time gap. This task attacks the query-time gap.
  They can run in parallel.
- **Task 21 (SIMD modernisation) owns broad SIMD backend work**.
  If a slice needs SVE/SVE2/AVX-512 backend, file it under Task 21
  and consume it here.
- **Task 60 (DiskANN RaBitQ storage format)** is orthogonal —
  storage format does not appear to be the cross-engine gap axis
  (pgvectorscale's index is *larger*), so Task 60 should be
  pursued for its own size-reduction merits, not as the
  cross-engine close.
- M5 is the local optimization host. Cloud confirmation belongs in
  Task 59 (DiskANN AWS Graviton tuning).
- Honor memory `feedback_dont_defer_safety_fixes` and
  `feedback_anti_pattern_b_unbounded_lifetime` in review.

## Stop Conditions

- Stop if the Phase 1 profile shows the gap is dominated by a
  factor outside the scan kernel (e.g. PG planner overhead,
  protocol layer cost) — escalate to a different task and shelve
  this one.
- Stop if the cumulative Phase 2 wins do not close at least half
  the measured 3.5× gap at L=64 after the top three P0 slices
  land. The remainder may be irreducible against this fixture +
  hardware combination; document and shelve rather than chase
  sub-5% slices indefinitely.
- Stop low-L work if the only apparent win is lowering rerank
  budget below the recall floor.
