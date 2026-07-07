# Task 168 packet 001 — outside-scan latency decomposition: artifact manifest

- Code under review: `da6101a00` ("Add query_prep / centroid_score /
  rescan_total stage timers to the IVF scan", branch
  `task-168-outside-scan-profile`, stacked on `task-145-topk-collect`
  head `d527d78d7`).
- Task bucket / packet: `reviews/task-168/001-outside-scan-profile/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port
  28818, db `tqvector_bench`. 2026-07-03.
- Timers only — no scan behavior change. Overhead A/B baseline = the
  Task 145 packet's `lazyheap` cell (`22411e3dd`, the immediate parent
  code state, same tables, ~40 min earlier same session).
- Fixtures: identical to the Task 145 packet 001 fixtures (isolated
  one-index-per-table, dbpedia 1536-dim, current defaults):
  `task145_default_real{10k,50k}`, `task143flip_default_real100k`,
  `task143_dense_1m`. No reloads — the suite's load steps skipped on
  row-count match.
- Suite: `task146-outside-scan-suite.json` (this packet) — the Task 145
  A/B config re-pointed at a packet-local artifact dir + truth cache
  (bespoke-config reason: same as that packet — fixed existing tables,
  ec_ivf lane only, registered default recall grid verbatim, latency
  [32,40], stage counters on).
- Runner: `target/release/ecaz` at `da6101a00`;
  `artifacts/timers/precheck-build-sha.log` records
  `ecaz_build_git_sha()` = `da6101a00...`; dylib shasum vs
  `target/release` in `artifacts/install-timers-dylib.log`. 17/17 steps
  succeeded (`artifacts/timers/suite-manifest.json`).

## Key result lines

### Gate 1 — timer overhead neutral

e2e mean vs the `lazyheap` cell (no new timers), all eight
scale×nprobe points: −6.0% .. +3.4%, no systematic direction (10k
+3.4/+3.1%, 50k −2.6/−6.0%, 100k −1.2/−0.5%, 1m −1.5/−2.9%) — within
this session's cross-run wobble. All 24 recall cells identical to the
lazyheap cell digit-for-digit.

### Gate 2 — the outside-scan decomposition (per-query µs, nprobe 32)

e2e = mean × scans; executor/gettuple = e2e − rescan_total. The
rescan-internal remainder (rescan_total − query_prep − centroid_score
− approximate_scan) equals the pre-existing probe_plan stage within
timer-nesting slop (100k: 141 vs 134 µs; 1m: 440 vs 432 µs), so the
rescan body is fully attributed with no unexplained slice.

| scale | e2e | approx scan | query_prep | centroid_score | probe_plan | executor/gettuple (share) |
|---|---|---|---|---|---|---|
| 10k  | 600  | 321  | 36 | 32  | ~49  | 162 (**27%**) |
| 50k  | 1120 | 745  | 35 | 73  | ~99  | 167 (15%) |
| 100k | 1610 | 1156 | 36 | 100 | ~134 | 177 (**11%**) |
| 1m   | 6660 | 5481 | 69 | 352 | ~432 | 318 (5%) |

(raw per-sweep numbers in `artifacts/timers/latency-*.log`
`[ivf-stage-counters]` lines; e2e means in
`artifacts/timers/results.jsonl`)

### Ranked follow-up shortlist (the Task 133 finding #4 answer)

1. **executor/gettuple: ~160–180 µs/query flat at ≤100k** (27% of e2e
   at 10k, 11% at 100k; 318 µs at 1m). This is k=10 heap-tuple fetches
   plus executor overhead per query — the largest non-scan slice at
   every scale ≤100k. Levers: index-only serving of the ORDER BY
   (avoid the heap fetch for tuples the executor discards), batched
   heap fetches. Substantial but interface-heavy.
2. **probe_plan grows with scale** (~134 µs at 100k, ~432 µs at 1m —
   6.5% of 1m e2e): dedup-pool alloc/clear + posting block-sequence
   build before the walk. Candidate: reuse/incremental block-sequence
   state across rescans of the same index.
3. **centroid_score scales with nlists** (32 → 352 µs from 10k → 1m,
   5.3% of 1m e2e): full-nlists exact f32 scoring per query. Candidates:
   SIMD/int8 centroid scoring (same rank-1 trick as the posting
   scorer), or partial selection (top-nprobe does not need a full exact
   ranking of all lists).
4. **query_prep is a non-lever**: ~36 µs flat (69 µs at 1m) — rotation
   + int8 query encode is already cheap; recorded as a source-grounded
   negative.

## Run log

- 2026-07-03 08:52: timers dylib `da6101a00` installed (shasum verified);
  suite run `artifacts/timers/`; post-run sha unchanged (precheck log +
  session check). No mid-run installs.
