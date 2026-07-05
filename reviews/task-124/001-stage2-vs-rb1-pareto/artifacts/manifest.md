# Task 124 packet 001 — stage2 pipeline vs rb1 champion: artifact manifest

- Measurement-only packet (no code change; the pipeline under test landed
  on main via the Task 130 keep-set, commits `9af6ba83e` + `fef5e20f6`).
- Binary for ALL new cells: installed dylib built at worktree HEAD
  `1dda8e589e3472c4f5c2476331e785a0add82f6a` (branch
  `task-124-stage2-pareto`), shasum `1326b1c0...` — see
  `install-stage2-dylib.log`. Code-identical to the Task 146/147 baseline
  binary `da6101a00` (`git diff da6101a00..1dda8e589 -- src/ crates/
  Cargo.toml Cargo.lock` is empty; interim commits are docs only), so the
  cited baseline cells compare cleanly.
  - Install-hazard note: the pre-existing installed dylib was
    `3f69d74c0` (co-located agent's Task 144 HNSW flip, which LACKS the
    Task 145 lazy-heap collect in the baselines' lineage); replaced
    before any cell ran; in-suite sha prechecks verify per run.
- Task bucket / packet: `reviews/task-124/001-stage2-vs-rb1-pareto/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port
  28818, db `tqvector_bench`, isolated one-index-per-table fixtures,
  dbpedia 1536-dim, seed 42. 2026-07-04.
- Question (re-baselined Task 124): does the landed in-engine 3-stage
  pipeline (rb1 coarse -> persisted TQ stage-2 over the width-50
  frontier -> exact f32 final rerank of 25) beat rb1 + heap_f32 width 50
  directly? Plus the apples-to-apples control Task 147 skipped
  (TQ coarse + the SAME heap_f32 rerank) and a no-stage2 width control
  (rb1@w25).
- Cells (fresh loads this packet):
  - **D (stage2)**: `storage_format=coarse_rerank`,
    `rerank_placement=index`, `rerank_format=turboquant`,
    `rerank_width=50`, `stage2_final_rerank_width=25` —
    `task124_stage2_real{10k,50k,100k}` + `task124_stage2_1m`.
  - **E (tqf32)**: `storage_format=turboquant`, `rerank=heap_f32`,
    `rerank_width=50` (dense auto per Task 143) —
    `task124_tqf32_real{10k,50k,100k}` + `task124_tqf32_1m`.
  - **F (rb1w25)**: `storage_format=rabitq`, `quant_bits=1`,
    `dense_posting_blocks=1`, `rerank=heap_f32`, `rerank_width=25` —
    `task124_rb1w25_real{10k,50k,100k}`.
- Baselines cited, not re-run (same binary lineage `da6101a00`):
  - **rb1@w50**: `reviews/task-147/001-density-pareto/artifacts/cells/`
    (+ `cells-1m/`).
  - **TQ pure default (no rerank)**:
    `reviews/task-146/001-outside-scan-profile/artifacts/timers/`.
- Suites: `task124-stage2-suite.json` (37 steps, ≤100k, 37/37
  succeeded) and `task124-stage2-1m-suite.json` (9 steps, winner tier).
  Bespoke-config reason: cross-storage-format pipeline matrix on the
  standard scales; registered ec_ivf default recall grid verbatim,
  latency [32,40], stage counters on.
- Runner: `target/release/ecaz`; suite env `PGHOST=/Users/peter/.pgrx
  PGPORT=28818` (runner-level connection; step-level flags come from the
  config as before).

## Key result lines (≤100k; 1m below)

### Recall@10 (nprobe 8→64)

| scale | cell | n8 | n16 | n24 | n32 | n48 | n64 |
|---|---|---|---|---|---|---|---|
| 10k | stage2 = tqf32 = rb1@w50 | 0.9812 | 0.9938 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 10k | rb1w25 | 0.9812 | 0.9906 | 0.9969 | 0.9969 | 0.9969 | 0.9938 |
| 50k | stage2 = tqf32 = rb1@w50 | 0.9437 | 0.9812 | 0.9875 | 0.9938 | 0.9938 | 0.9938 |
| 50k | rb1w25 | 0.9344 | 0.9688 | 0.9750 | 0.9812 | 0.9812 | 0.9812 |
| 100k | stage2 = tqf32 = rb1@w50 | 0.8156 | 0.8750 | 0.9187 | 0.9375 | 0.9563 | 0.9719 |
| 100k | rb1w25 | 0.8063 | 0.8656 | 0.9094 | 0.9281 | 0.9469 | 0.9625 |

Two recall facts:

1. **stage2, tqf32, and rb1@w50 are recall-IDENTICAL at all 18 points.**
   The exact width-50 frontier determines recall; neither the coarse
   payload (1-bit vs 4-bit — the Task 147 masking fact, now confirmed
   head-on under the SAME rerank) nor inserting the TQ stage-2 reducer
   before a width-25 exact stage changes which truth rows surface.
2. **Plain width-25 exact rerank (rb1w25) LOSES recall** (−0.3..−1.3 pp
   across the grid): the rb1 coarse ordering alone puts truth rows in
   ranks 26–50 often enough to matter. The TQ stage-2 reducer re-orders
   the width-50 frontier well enough that its top-25 exact set keeps
   them — the reducer has a real job; cutting exact width without it is
   not free.

### Latency (mean / p95 ms, nprobe 32 and 40)

| scale | stage2 (D) | tqf32 (E) | rb1w25 (F) | rb1@w50 (base) |
|---|---|---|---|---|
| 10k n32 | 0.65 / 0.77 | 0.69 / 0.84 | 0.58 / 0.67 | 0.67 / 0.78 |
| 10k n40 | 0.71 / 0.85 | 0.72 / 0.87 | 0.66 / 0.73 | 0.71 / 0.80 |
| 50k n32 | 1.17 / 1.34 | 1.10 / 1.30 | 1.00 / 1.11 | 1.12 / 1.27 |
| 50k n40 | 1.31 / 1.63 | 1.23 / 1.41 | 1.18 / 1.28 | 1.33 / 1.56 |
| 100k n32 | **1.52 / 1.70** | 1.58 / 1.97 | 1.46 / 1.70 | 1.58 / 1.81 |
| 100k n40 | **1.71 / 1.91** | 1.78 / 2.03 | 1.66 / 1.88 | 1.83 / 2.13 |

- stage2 vs rb1@w50 at matched (identical) recall: −4% mean / −6% p95
  at 100k n32, −6.6% / −10% at n40; parity at 10k; ~+4% at 50k n32
  (within this session's observed ±4–6% 50k wobble). The modest win
  grows with scale — 1m tier below is the deciding cell.
- tqf32 vs rb1@w50 (the apples-to-apples density control): recall
  identical, latency parity-to-slightly-worse at ≤100k warm. The 4×
  coarse-byte advantage of rb1 is NOT yet decisive at ≤100k e2e; the
  decisive deltas at these scales are storage (below) and the rerank
  stage. 1m is where Task 147 saw posting bytes matter.
- rb1w25 is fastest but NOT at matched recall — dominated on the
  recall axis; excluded from the 1m tier.

### Rerank-stage split (100k, n32, per-sweep of 32 queries)

| cell | exact_rerank | rerank_payload_decode | rerank_payload_score |
|---|---|---|---|
| stage2 | 7.09 ms | 5.45 ms (TQ sidecar) | 0.13 ms |
| tqf32 | 5.05 ms | 4.67 ms (heap f32) | 0.13 ms |
| rb1w25 | 3.25 ms | 2.98 ms (heap f32) | 0.06 ms |
| rb1@w50 (147) | 5.82 ms | — | — |

The TQ stage-2 payload cost is ~98% DECODE, ~2% score
(5.45 vs 0.13 ms) — the stage-2 e2e win at 100k comes despite a more
expensive rerank block. **TQ sidecar payload decode is the top
optimization target** if this pipeline is pursued (exactly the Task 124
"drill into TQ-specific bottlenecks" clause: decode, not scoring).

### Storage (index size)

| scale | stage2 | tqf32 | rb1w25 | rb1@w50 |
|---|---|---|---|---|
| 10k | 11.4 MiB | 9.0 MiB | 3.3 MiB | 3.3 MiB |
| 50k | 53.2 MiB | 41.6 MiB | 13.5 MiB | 13.5 MiB |
| 100k | 104.6 MiB | 81.7 MiB | 25.5 MiB | 25.5 MiB |

The stage2 sidecar costs 4.1× rb1's index (persisted TQ payload ≈ the
full 4-bit code per row on top of the 1-bit coarse). rb1@w50 keeps the
storage crown by a wide margin; stage2's value must come from latency
(marginal at ≤100k) or from halving exact heap fetches (25 vs 50/query
— the IO-sensitive Phase 6 axis).

### 1m tier (990k anchor split, 9/9 steps succeeded)

Recall@10:

| cell | n8 | n16 | n24 | n32 | n48 | n64 |
|---|---|---|---|---|---|---|
| stage2 | 0.8438 | 0.9187 | 0.9375 | 0.9563 | 0.9719 | 0.9781 |
| tqf32 | 0.8500 | 0.9250 | 0.9437 | 0.9625 | 0.9781 | 0.9844 |
| rb1@w50 (147) | 0.8417 | 0.9250 | 0.9417 | 0.9667 | 0.9750 | 0.9792 |

Latency (mean / p95 ms):

| cell | n32 | n40 |
|---|---|---|
| stage2 | 5.50 / 6.59 | 6.30 / 7.52 |
| tqf32 | 5.92 / 7.56 | 7.20 / 8.86 |
| rb1@w50 (147) | 6.21 / 7.59 | 6.63 / 7.86 |

Index size: stage2 **1003.3 MiB**, tqf32 784.8 MiB, rb1@w50 226.6 MiB.

Stage split at n32 (per-sweep of 32 scans; rb1@w50 line scaled from its
16-scan sweep): approximate_scan stage2 4.13 ms/query vs tqf32 4.58 vs
rb1@w50 4.63; exact_rerank stage2 0.51 ms/query (incl. TQ decode 0.39)
vs tqf32 0.35 vs rb1@w50 0.49.

1m reading — the ≤100k recall-identity does NOT fully carry:

- **stage2 gives up 0.3–1.0 pp recall at n≥24** (n32: 0.9563 vs
  rb1@w50's 0.9667). At matched recall the latency win disappears:
  stage2 needs ~n40 (0.9719 @ 6.30 ms) to clear rb1@w50's n32 point
  (0.9667 @ 6.21 ms) — roughly pareto-EQUIVALENT warm, at 4.4× the
  index size. The raw −11% at n32 is not a matched-recall comparison.
- **tqf32 vs rb1@w50 stays recall-equivalent within noise** (32-query
  CI ~±2 pp; deltas −0.4..+0.8 pp) with mixed latency (−5% at n32,
  +9% at n40) and 3.5× the storage. The controlled density conclusion:
  with the SAME exact rerank, the 4-bit vs 1-bit coarse payload is
  ~recall/latency-neutral warm; **the Task 147 rb1-vs-TQ-default win
  was primarily the rerank stage, and density's durable payoff is
  storage** (3.2–3.5× smaller) plus a latency edge that only appears
  at deeper sweeps.

### Verdict

**No warm-cache promotion case.** rb1 + heap_f32 width 50 (the Task 147
champion) survives all three challengers on the warm pareto once recall
is matched, and keeps a 3.2–4.4× storage advantage:

- **D (stage2@25)**: recall-identical and −4..−10% latency at ≤100k,
  but at 1m it pays 0.3–1.0 pp recall and lands pareto-equivalent at
  matched recall — while costing 4.4× the index bytes. **Iterate, not
  promote**: its two live rationales are (1) it halves exact heap
  fetches per query (25 vs 50) — the Phase 6 IO-sensitive/cold-cache
  axis, still unmeasured, is now the deciding evidence for the whole
  Task 124 premise; (2) TQ payload decode is 98% of the stage-2 cost
  (5.45 of 5.58 ms/sweep at 100k) — a decode-path optimization could
  tilt the warm comparison before Phase 6 is even run.
- **E (tqf32)**: closes the Task 147 apples-to-apples gap — density
  alone (holding rerank fixed) is recall-neutral and roughly
  latency-neutral warm; rb1's structural win is storage. No reason to
  run a TQ coarse stage under rerank at 3.5× the bytes.
- **F (rb1@w25)**: cheapest and fastest but loses 0.3–1.3 pp recall at
  every scale — naive width reduction is not free; if 25-wide exact
  fetches are ever needed (IO regimes), the stage-2 reducer (D) is the
  recall-preserving way to get there at ≤100k scales.

Follow-ups this evidence motivates (not started here):

1. **Phase 6 cold-cache A/B** (rb1@w50 vs stage2@25 vs TQ no-rerank at
   100k/1m) — the fetch-count rationale is now the only path to a
   stage2 promotion; TQ-no-rerank's zero-heap-fetch niche gets tested
   by the same run.
2. **TQ sidecar decode optimization** (98% of stage-2 payload cost is
   decode, not scoring) — bounded, well-attributed target if Phase 6
   shows promise.
3. The rb1 promotion-matrix task (Task 147 follow-up) should cite this
   packet's E-cell as the controlled density evidence.

## Run log

- 2026-07-04 ~16:2x: first ≤100k launch failed at runner startup
  (`both host and hostaddr are missing` — runner-level PG connection
  needs `PGHOST`; prior sessions had it exported). Relaunched with
  `PGHOST=/Users/peter/.pgrx PGPORT=28818`; 37/37 succeeded
  (`artifacts/cells/suite-manifest.json`).
- 2026-07-04 ~17:0x: first 1m launch failed on a config authoring error
  (`ec_real_1m_*` staged names do not exist; the staged 1m corpus is the
  990k `ec_real_ann_benchmarks_anchor_*` split, as in the Task 147 1m
  cell); paths fixed, relaunched, 9/9 succeeded
  (`artifacts/cells-1m/suite-manifest.json`).
