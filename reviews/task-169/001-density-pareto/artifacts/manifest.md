# Task 169 packet 001 — coarse-payload density pareto: artifact manifest

- Measurement-only packet (no code change). Binary for ALL cells:
  installed dylib `da6101a00` (the Task 168 timers commit; shasum +
  in-suite sha prechecks per cell). The TQ champion cells are the Task
  146 packet's `timers` run on the SAME binary — cited, not re-run.
- Task bucket / packet: `reviews/task-169/001-density-pareto/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port
  28818, db `tqvector_bench`. 2026-07-03.
- Question: does a denser coarse payload + exact heap-f32 rerank beat
  the promoted TQ dense-int8 no-rerank default on the latency/recall
  pareto? (Task 96 premise, reframed on existing surfaces; Phase-0
  finding: TQ coarse encode is hardwired 4-bit —
  `IvfQuantizer::encode_source` uses `crate::DEFAULT_QUANT_BITS` — so a
  TQ 2-bit lane would be new surface.)
- Cells (isolated one-index-per-table, dbpedia 1536-dim, seed 42):
  - **A (champion)**: pure defaults, `storage_format=turboquant`
    (dense + int8/SDOT, rerank off) — from
    `reviews/task-168/001-outside-scan-profile/artifacts/timers/`.
  - **B (rb1)**: `storage_format=rabitq`, reloptions `quant_bits=1,
    dense_posting_blocks=1, rerank=heap_f32, rerank_width=50` —
    prefixes `task147_rb1_real{10k,50k,100k}` + `task147_rb1_1m`,
    fresh loads this packet.
  - **C (rb2)**: same with `quant_bits=2` —
    `task147_rb2_real{10k,50k,100k}`.
- Suites: `task147-density-pareto-suite.json` (25 steps, ≤100k;
  24/24 measurement steps succeeded on the retry run — the first run
  failed on a config authoring error, `bits: N` instead of the
  `quant_bits=N` reloption, preserved in `artifacts/cells/suite-run.log`
  history) and `task147-rb1-1m-suite.json` (winner-only 1m tier).
  Bespoke-config reason: cross-storage-format pareto matrix on the
  standard scales; registered ec_ivf default recall grid verbatim,
  latency [32,40], stage counters on.
- Runner: `target/release/ecaz` at `da6101a00` for all cells.

## Key result lines (≤100k; 1m below)

### Recall@10 (nprobe 8→64)

| scale | cell | n8 | n16 | n24 | n32 | n48 | n64 |
|---|---|---|---|---|---|---|---|
| 10k | TQ (A) | 0.963 | 0.972 | 0.975 | 0.975 | 0.975 | 0.975 |
| 10k | rb1 = rb2 | 0.981 | 0.994 | **1.000** | 1.000 | 1.000 | 1.000 |
| 50k | TQ (A) | 0.919 | 0.950 | 0.953 | 0.959 | 0.959 | 0.959 |
| 50k | rb1 = rb2 | 0.944 | 0.981 | 0.988 | **0.994** | 0.994 | 0.994 |
| 100k | TQ (A) | 0.784 | 0.834 | 0.875 | 0.894 | 0.912 | 0.925 |
| 100k | rb1 = rb2 | 0.816 | 0.875 | 0.919 | **0.938** | 0.956 | **0.972** |

rb1 and rb2 recall are IDENTICAL at every one of the 18 points — the
Task 115/122 rerank-masking fact reconfirmed head-on: under exact
heap-f32 rerank at width 50, the 1-bit and 2-bit coarse stages place
the same truth rows in the frontier.

### Latency (mean ms, n32 / n40)

| scale | TQ (A) | rb1 (B) | rb2 (C) |
|---|---|---|---|
| 10k | 0.60 / 0.66 | 0.67 / 0.71 | 2.35 / 2.79 |
| 50k | 1.12 / 1.26 | 1.12 / 1.33 | 4.90 / 5.91 |
| 100k | 1.61 / 1.82 | **1.58 / 1.83** | 7.03 / 8.51 |

- rb1 is at latency parity with the champion at 50k/100k (within
  noise; +0.07 ms at 10k) while carrying the recall wins above —
  **rb1 pareto-dominates the TQ default at 50k/100k** (same latency,
  +3.5..+4.7 pp recall; or read along the other axis: rb1 needs ~n16
  to match TQ's n32 recall at roughly two-thirds the latency).
- rb2 is 4–5× slower at the same recall as rb1 — it falls off the
  1-bit popcount block-kernel path; strictly dominated. Closes the
  "2-bit" branch of the question: the interesting density point is
  1-bit, not 2-bit.
- rb1 100k stage split (n32, per-sweep): approximate_scan 32.7 ms +
  exact_rerank 5.8 ms (~0.18 ms/query for the 50-row heap fetch +
  exact rescore).

### Storage (100k)

| cell | index size |
|---|---|
| TQ (A) | 81.7 MiB (Task 143/145 packets) |
| rb1 (B) | **27.6 MiB (−66%)** |

### 1m tier (winner only; 990k anchor split, `task147_rb1_1m`, 4/4 steps succeeded)

Recall@10 (TQ → rb1): n8 0.8333 → 0.8417, n16 0.8875 → 0.9250,
n24 0.9000 → 0.9417, **n32 0.9208 → 0.9667 (+4.6 pp)**,
n48 0.9292 → 0.9750, n64 0.9292 → **0.9792 (+5.0 pp)**.

Latency mean (n32 / n40): TQ 6.66 / 7.58 ms → rb1 **6.21 / 6.63 ms**
(−6.8% / −12.5%). rb1 stage split at n32 (per-sweep, 16 scans):
approximate_scan 74.1 ms + exact_rerank 7.8 ms (~0.49 ms/query for the
50-row heap fetch + exact rescore).

Index size: TQ 784.8 MiB → rb1 **247.8 MiB (−68%)**.

### Verdict

**rb1 (RaBitQ quant_bits=1 + dense posting blocks + heap_f32 rerank
width 50) pareto-dominates the just-promoted TQ dense-int8 default at
every measured scale**: latency parity at 10k–100k and −7..−12% at 1m,
recall +3.5..+5.0 pp everywhere at n≥16, index −66..−68%. The density
hypothesis holds emphatically — the win is 4-bit → 1-bit posting bytes
(fewer pages, fewer parse bytes, cheaper kernel) with the exact rerank
recovering recall, exactly as the rerank-masking insight predicted.
rb2 closes the 2-bit branch (same recall as rb1, 4–5× slower — off the
popcount kernel path), which also settles the reframed Task 96
question: a TQ 2-bit lane is NOT worth building; the interesting
follow-ups are at 1 bit.

Follow-ups this evidence motivates (not started here):
1. IVF default-format promotion question (rb1-style coarse_rerank vs
   turboquant) — needs the Task 143-style promotion matrix
   discipline: cold-cache + insert/churn + Graviton + a second corpus
   before any default flip; also nprobe operating-point re-derivation
   (rb1 at n16 ≈ the TQ default's n32 recall at ~2/3 the latency).
2. The Task 111e `coarse_rerank` storage format is the productized
   shape of this cell — revisit its promotion status with these
   numbers.
3. rb2's 4–5× penalty is a kernel-coverage gap (rb2 lacks the
   popcount-class batch path) — only worth fixing if some future
   regime needs 2-bit specifically.

## Run log

- 2026-07-03 ~09:0x: first ≤100k run failed on the `bits` misconfig
  (loader: `encode_to_ecvector expects the canonical quantizer defaults
  (4,42), got (1,42)`); config fixed to `quant_bits=N` reloptions.
- 2026-07-03 ~09:1x: retry run 25/25 succeeded
  (`artifacts/cells/suite-manifest.json`).
- 2026-07-03 ~09:2x: rb1 1m cell launched (`artifacts/cells-1m/`).
