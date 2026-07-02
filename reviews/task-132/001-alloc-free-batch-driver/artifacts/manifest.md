# Task 132 Alloc-Free Batch Driver + Dim-Tiling Decision Artifacts

- task bucket: `reviews/task-132/001-alloc-free-batch-driver`
- code commits:
  - `87eb5ad13` Add Task 132 batch-tiled width microprofile harness
  - `c5201bffc` Replace whole-batch tiled NEON kernel with alloc-free octet-sliced driver
- base: `a05babf74` (Task 133 stage counters; task-125 int16 scorer unchanged)
- lane: local PG18 (Homebrew 18.3), Apple M5 Pro — 6 "Super" cores @128 KB L1D +
  12 "Performance" cores @64 KB L1D
- fixture: staged real corpus (dbpedia 1536-dim, SHAs in task-133 packet manifest),
  `ec_ivf`, `storage_format=turboquant`, bits=4, seed=42, nprobe=32, k=10
- installed backend for the e2e A/B: release, `ecaz_build_git_sha() = c5201bffc…`
  (recorded in `suite-manifest-slice3.json` backend block; runner git sha ditto)
- isolation: `task133_tq_ivf_real*` prefixes dropped and fully reloaded before the
  candidate run (extension recreate had invalidated embedding columns)

## Artifacts

- `width-profile-before.log` — release ns/candidate sweep at widths
  8..1024, dim 1536, on `87eb5ad13` (old whole-batch tiled kernel with hot-path allocs)
- `width-profile-after.log` — same sweep on `c5201bffc` (alloc-free driver)
- `width-profile-after-64kb-qos.log` — same sweep under `taskpolicy -c background`
  (steered to the 64 KB-L1D Performance cluster; see caveats)
- `install-slice3.log` — `cargo pgrx install --release` log
- `suite-manifest-slice3.json` / `results-slice3.jsonl` + per-step logs — e2e
  IVF suite (task-133 config) on the slice-3 dylib
- baseline for the e2e comparison: task-133 packet
  `results-latency-quiet.jsonl` (dylib `a05babf74`, same session)

## Key results

Microbench (ns/candidate, dim 1536, Super cores, release):

| width | before | after | delta |
|---|---|---|---|
| 8 | 375.1 | 243.1 | −35% |
| 16 | 183.7 | 143.2 | −22% |
| 24 | 158.6 | 128.1 | −19% |
| 32 | 124.0 | 121.7 | −2% |
| 39 | 129.8 | 122.2 | −6% |
| 48 | 126.9 | 118.0 | −7% |
| 64 | 125.2 | 117.2 | −6% |
| 128 | 120.6 | 119.5 | ~0 |
| 256 | 131.8 | 120.4 | −9% |
| 512 | 133.2 | 121.1 | −9% |
| 1024 | 136.4 | 121.0 | −11% |

e2e IVF (candidate dylib `c5201bffc` vs quiet baseline `a05babf74`, same session):

- recall **identical**: 0.9734 / 0.9521 / 0.8969 (bit-exactness holds e2e)
- kernel: 23.86/39.63/40.38 ms vs 23.99*/39.72/39.71 ms — **neutral within noise**
  (*first-run figure; quiet-baseline 10k kernel not separately logged)
- mean latency reads 0.91/1.84/3.01 vs 1.09/1.99/3.21 ms, but the candidate ran on
  freshly rebuilt tables — the delta is confounded with physical layout and is
  **not claimed** as an alloc-fix win.
- storage: unchanged (same index B/row as task-133 run)

64 KB-L1D probe (`taskpolicy -c background`, Performance cluster):

- absolute ~2.4–2.6× slower across ALL widths (clock/QoS, expected)
- curve **flat from width 32 up** (289→273 ns across 32..1024, ±7% jitter) — no
  width-dependent degradation of the kind L1D thrash would produce
- caveats: macOS QoS steering is not verified core placement; no hardware cache
  counters; higher scheduler noise at background QoS. Supporting evidence only.

## Dim-tiling decision (Task 132 gate)

Shelve the dimension-tiling lever, on evidence:

1. Arithmetic: the task-125 i16 LUT is `dim × 32 B` = 48 KiB at dim 1536 —
   **under the 64 KB Graviton L1D for every dim ≤ 2048**. The scenario tiling was
   designed for (LUT > L1D) no longer exists on any current target/fixture.
2. Apple e2e: neutral-to-worse twice (task-125 packet 002 candidate-tiling; this
   packet's kernel-neutral result).
3. 64 KB-class cores: flat width curve (above), no residency cliff at 48 KiB.

What ships instead: the alloc-free octet-sliced driver — removes all three
reviewer-flagged hot-path heap allocations, deletes the whole-batch NEON kernel
(−2 `unsafe`, net −93 lines), wins 6–35% ns/candidate at graph-AM-sized widths
and 9–11% at slab widths in the microbench, bit-exact, kernel/recall/storage
neutral e2e on IVF.

Open (recorded, not blocking): a Graviton lane run remains the formal cross-check
if a dim > 2048 fixture ever becomes a target; noted in request.md.
