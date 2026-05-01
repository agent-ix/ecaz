# Task 29d: DiskANN Pre-Landing Performance Sweep

Status: planned, blocks merge of `task29-diskann-initial-tuning`
Owner: coder1 / runtime-index track
Backstory: `review/11105-task29-release-latency-refresh/feedback.md`

## Goal

Close the three remaining performance questions before the Task 29
landing slice merges:

1. Settle whether the build-side heap frontier is a real regression
   (released or buried).
2. Close the L=64 scan-latency gap vs pgvectorscale (currently
   2.6×).
3. Reduce DiskANN build cost from ~71 s toward the
   pgvectorscale/HNSW reference (~5–6 s) — measurement-driven, with
   a stop condition.

The first two are small, focused experiments. The third is the
larger scope and is structured the same way Task 29c was:
profile-first, attack the biggest fixable contributor, stop at
diminishing returns. The user direction stands: don't set
unreasonable expectations; exhaust within reason; have honest
reference numbers in hand for the merge discussion.

## Order of operations

1. **29d-1 (½ day): Build heap-frontier release-mode A/B.** Resolves
   a deliberately-deferred round-2 open question. Quick yes/no.
   Likely informs 29d-3 since the build-side frontier is one of the
   candidate optimizations there.
2. **29d-2 (1–2 days): L=64 scan latency profile + targeted fix.**
   Profile the constant-factor gap; if a single-source attack
   exists, take it. Otherwise document and move on.
3. **29d-3 (1–3 weeks): Build performance attack with stop
   condition.** The bigger investment. Measurement-driven.
4. **Final readiness packet.** Refreshed full sweep (recall +
   latency + storage + build, both engines) at the post-29d head;
   sign-off review.

Each sub-task gets its own packet. Land changes one at a time so
deltas are clean.

## 29d-1 — Build heap-frontier release-mode A/B (½ day)

### Background

Packet `11101` reverted the build-side heap-frontier experiment
(`d2e0e9fc`) after measuring a 12% regression. That measurement
was taken **before** the debug-vs-release discovery in packet
`11102`, so it ran on a debug-installed extension. The same shape
of change on the scan side (`27bb6af8`) was a clear release-mode
win. Whether the build-side asymmetry is real or a debug artifact
is unresolved.

### Procedure

1. Cherry-pick `d2e0e9fc` (heap-frontier build) and `36f0c3d5`
   (the truncation fix that followed it) onto current HEAD. Stage
   only — don't push.
2. `cargo pgrx install --release` the cherry-picked head into the
   local PG18 install tree.
3. Run the same isolated real-10k DROP+CREATE INDEX measurement
   used in packets `11102`/`11104` against the
   `task29c_phase_profile_corpus` table. Capture full structured
   timing.
4. Compare to the active-mask baseline (70.678 s total, 67.571 s
   core graph time).

### Decision matrix

| Outcome | Action |
|---|---|
| Wins by ≥ 5% | Re-land the cherry-pick. Update packet 29d-3 baseline. |
| Within ±5% | Leave reverted. Document as "no benefit, no regression in release mode." |
| Loses by ≥ 5% | Leave reverted. Asymmetry is real (probably build-side frontier sizes are too small to amortize heap overhead). |

### Validation

- `cargo test --lib am::ec_diskann::vamana -- --nocapture` (the
  build-recall sanity test stays green).
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`.

### Out of scope

- Inventing a hybrid frontier (e.g., heap above some threshold,
  linear below). If the experiment shows an interesting size-
  dependent crossover, file as a follow-up; do not land within
  29d-1.

## 29d-2 — L=64 scan latency parity with pgvectorscale (1–2 days)

### Background

Packet `11105` measured ec_diskann at 9.19 ms / pgvectorscale at
3.56 ms at L=64 on the same real-10k corpus. That's the cleanest
apples-to-apples constant-factor signal in the comparison (both
engines rerank essentially everything they have at low L, so the
asymmetry in `query_rescore` defaults doesn't apply here).

The 2.6× gap at L=64 is also the gap that explains why
pgvectorscale wins at low L overall. If we close it, ec_diskann
becomes faster than pgvectorscale across the entire L sweep.

### Phase 2A — Profile (1 day)

Capture a `cargo flamegraph` (or `perf record`) of the
release-mode ec_diskann scan path under the same workload that
produced the 9.19 ms number. Use `EXPLAIN (ANALYZE, BUFFERS)` on
single representative queries to identify any per-query overhead
that isn't visible from average latency.

Key candidates to identify in the profile:

- **Per-amrescan setup cost.** The SRHT rotation + sign-bit pack +
  metadata page read happens once per query. At ~16 µs theoretical
  it should be a small fraction of 9 ms, but verify.
- **Per-visit page read cost.** With L=64 and graph_degree=32
  (so ~5–10 visits per query at default settings), each visit
  triggers reads of the picked node's tuple + up to 32 neighbor
  tuples. If neighbors are not co-located with the parent on the
  same page, every visit could incur 30+ random page accesses.
- **Hamming popcount in the prefilter loop.** Should be ~30 ns
  per node per the SIMD codegen check from packet `11100`. Verify.
- **Heap fetch + exact rerank.** rerank_budget=64 means up to 64
  heap fetches per query. At ~50 µs warm-cache fetch each, that's
  3.2 ms — a meaningful fraction of 9 ms.
- **Per-amgettuple overhead.** The result buffer expansion in
  `expand_scan_results_with_bound_heap_tids` runs in `amrescan`
  rather than streaming via `amgettuple`, so each query
  materializes the full top-k. Measure whether this is a
  meaningful chunk vs the actual scan work.

Capture the top 10 hot stack frames as a packet artifact.

### Phase 2B — Targeted fix (½–1 day, conditional)

If the profile identifies a single dominant contributor (≥ 30% of
9 ms), attack it. Likely candidates and their fixes:

- **Heap fetch dominates** → consider whether the existing
  `binary_words` sidecar can stand in as a finer-grain rerank for
  candidates that are tied or near-tied at the popcount level,
  reducing heap-fetch count for the bulk of queries. (This is a
  meaningful design change — discuss before landing.)
- **Per-page read overhead** → check whether `PersistedGraphReader`
  re-reads the same page multiple times in a single scan; cache
  decoded tuples for the duration of the scan if so.
- **Result-buffer materialization** → if `expand_scan_results_with_bound_heap_tids`
  is allocating per-query, switch to a scratch buffer reused across
  rescans.
- **No single dominant contributor** → document the breakdown and
  move on. Constant-factor work spread across many small functions
  is harder to attack and may need its own task.

### Stop condition

If Phase 2B's targeted fix doesn't move L=64 mean below 6 ms (i.e.
within 1.7× of pgvectorscale), document the residual breakdown and
land what's measured. The merge discussion can frame this as
"closed the gap to within Xx" rather than "achieved parity."

### Validation

- Same `cargo bench latency --sweep 64,128,200,400,800` sweep
  used in packet `11105`, on the same prefix.
- Recall must stay at ≥ 0.996 across the sweep.
- HWM must stay below 75 MiB across the sweep.

## 29d-3 — Build performance attack (1–3 weeks)

### Background

After `11104`'s active-mask prune cleanup, the release-mode
real-10k DiskANN build is 70.678 s. Reference engines on the same
corpus: pgvectorscale 5.82 s, ec_hnsw 5.23 s. Gap is ~12×.

The structured timing from packet `11102` localized the cost:

- core medoid: 1.566 s
- core graph: 67.571 s **← this is the chunk**
  - pass 0 elapsed: 20.737 s (post-active-mask)
  - pass 1 elapsed: 46.832 s (post-active-mask)
- core persist: 0.014 s
- write pages: 0.059 s

Distance calls per the `11102` counters (pre-active-mask, but
shape unchanged):

- pass 1 greedy_search: 12.86M distance calls in 21.0 s
- pass 1 robust_prune: 17.84M distance calls in 6.9 s
- pass 1 backlink: 0.61M distance calls in 9.9 s

So pass-1 work is dominated by **31M distance evaluations**
totaling ~38 s. Each distance call is an exact f32 inner product
over 1536-d source vectors — dominated by the `source_inner_product_distance`
helper in `ambuild.rs:322-334`, which is roughly 1500 multiplies
+ 1500 adds per call.

### Phase 3A — Profile to confirm the bottleneck (2–3 days)

Capture a `cargo flamegraph` of the release-mode build to confirm
the distance evaluation dominance and identify whether:

- Distance calls hit the SIMD-optimized `source_inner_product_distance`
  path inside the build closure, or fall to a scalar implementation.
- The `source_refs[a as usize]` Vec indexing inside the closure is
  cache-friendly.
- There are repeated `read_node` decodes from `PersistedGraphReader`
  during build (would explain part of pass-1 latency beyond just
  distance work).

This profile sets the priority for Phase 3B.

### Phase 3B — Attack candidates (2–10 days, ranked by profile)

In rough order of expected payoff (subject to revision after
Phase 3A):

1. **SIMD the build distance.** `source_inner_product_distance` at
   `ambuild.rs:322-334` is a scalar `iter().zip().map().sum::<f32>()`
   that the compiler may or may not auto-vectorize. Check the
   release-mode codegen (same `cargo asm` approach as `11100`'s
   Hamming check). If not SIMD'd, transliterate to the AVX2
   implementation that pgvectorscale's `inner_product_unoptimized`
   uses. Expected payoff: ~3-5× per distance call → ~25-30 s off
   pass-1.
2. **Cache decoded tuples during build.** `PersistedGraphReader::read_node`
   decodes from raw bytes on each call. The build calls it many
   times per pivot during greedy descent. An in-memory
   `HashMap<ItemPointer, VamanaNodeTuple>` for the build duration
   eliminates the redundant decode work. (For 10k rows the entire
   decoded graph fits in memory; for 10M+ a bounded LRU.)
3. **Distance reuse between greedy and prune.** The greedy descent
   for pivot P computes dist(P, candidate) for many candidates;
   robust_prune then computes dist(pivot_star, v) for many
   candidate pairs. Some of these distances may overlap and be
   cacheable for the duration of one pivot's processing.
4. **Pass-0 / pass-1 algorithm tuning.** The two-pass [α=1.0, α=1.2]
   schedule may have headroom. pgvectorscale's progressive-α loop
   inside a single pass (max_factors[] tracking) is a known
   alternative shape. Mostly an algorithmic experiment, not a
   constant-factor fix.

Each attack lands as its own packet with before/after timing on
the same prefix.

### Stop condition

Stop when **either**:

- Build cost is within **3× of the strongest reference** (≤ 17.5 s
  given pgvectorscale's 5.82 s). At that point we have a defensible
  landing number.
- The next attack candidate would take >5 days and Phase 3A profile
  data suggests <15% of remaining cost. At that point, we've
  exhausted single-process options within reason — the next ask
  would be parallel build (Task 29e), which is a separate scope.

Document what was attempted and what remains either way. The
landing decision keys off whichever stop fires first.

### Validation

- Build correctness: every per-attack packet must pass the existing
  `cargo test --lib am::ec_diskann::vamana::build_recall_at_10_meets_baseline`
  + `cargo pgrx test pg18 test_ec_diskann_*`.
- Recall regression check: rerun `ecaz bench recall` on real-10k at
  L=64,200,800 after the final attack lands. Recall must stay ≥
  0.996 across the sweep.
- Index size: must stay within 5% of the post-`11104` baseline
  (4824 kB).

## Final 29d landing readiness packet

After 29d-1, 29d-2, and 29d-3 all land:

- Refresh the full release-mode sweep: ec_diskann + pgvectorscale +
  ec_hnsw on the same corpus, same release-installed extensions,
  same hardware. Recall + mean latency + p50/p95/p99 + index size +
  build cost.
- Compare to packet `11105` numbers. Document what moved and by
  how much.
- Update parent task `plan/tasks/29-diskann-initial-tuning.md` with
  29d outcome links.
- Open round-4 review feedback file. After sign-off, merge.

## Out of scope for 29d (deliberate)

- **Parallel DiskANN build.** That's Task 29e if 29d-3's stop
  condition fires before reaching the reference range. Don't
  speculate on its scope today.
- **Tuple format changes.** `tuple.search_code` / `binary_words` /
  `grouped_codebook_head` all stay where they are. Any storage
  layout change is a separate task with its own rebuild path.
- **GPU-accelerated build.** ADR-046 follow-up; not relevant here.
- **Streaming amgettuple shape.** Round-2 noted the one-shot
  `amrescan` materialization; out of scope for 29d unless 29d-2's
  profile says it's the dominant L=64 contributor.

## Acceptance criteria

- One packet per sub-task (29d-1, 29d-2, 29d-3 — 29d-3 may
  contain multiple sub-packets per attack landed).
- One final landing-readiness packet with the post-29d full sweep.
- Build cost within 3× of the strongest reference, **or** explicit
  documented stop with the residual breakdown.
- L=64 latency within 1.7× of pgvectorscale, **or** explicit
  documented stop with the residual breakdown.
- All existing tests still pass (currently 167 in `am::ec_diskann::*`
  + 19 pgrx callbacks).
- `cargo clippy --features pg18 -D warnings` stays clean.
- Round-4 sign-off review feedback file in
  `review/<final-packet>/feedback.md`.
