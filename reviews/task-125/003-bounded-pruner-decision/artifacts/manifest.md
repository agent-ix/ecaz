# Task 127 Bounded-Pruner Decision Artifacts

- task bucket: `reviews/task-125/003-bounded-pruner-decision` (Task 127 is a
  task-125-bucket concern per the closeout review)
- measurement commit: `23f3c752c` (dylib `c5201bffc`, recorded in
  `suite-manifest.json` backend block with runner git sha)
- removal commit: (this packet's code change — see request.md)
- lane: local PG18 (Homebrew 18.3), Apple M5 Pro
- fixture: staged real corpus (dbpedia 1536-dim; SHAs in task-133 packet
  manifest), `ec_ivf`, `storage_format=turboquant`, bits=4, seed=42,
  **`rerank=heap_f32`, `rerank_width=100`** (the only config where the bounded
  batch scorer activates), nprobe=32, k=10
- suite config: `task127-pruner-ab-suite.json` (bespoke: prune on/off A/B via
  `ec_ivf.posting_bound_prune` session GUC × 3 scales; standard configs cannot
  express the A/B axis)
- isolation: fresh `task127ab_tq_ivf_real{10k,50k,100k}_heap` one-index-per-prefix
  tables loaded by this suite
- runner: `target/release/ecaz bench suite run --config ... task127-pruner-ab-suite.json`
- timestamp: 2026-07-02 (results.jsonl)

## Key results (`results.jsonl`; logs `latency-heap-{scale}-prune-{on,off}.log`)

Activation confirmed — prune fractions from `[block-kernel-counters]`:

| scale | pruned | kept | fraction |
|---|---|---|---|
| 10k | 186274 | 3301 | 98.3% |
| 50k | 320421 | 7698 | 97.7% |
| 100k | 316516 | 7049 | 97.8% |

Recall parity exact (prune on ↔ off): 1.0000 / 0.9766 / 0.9219.

Latency (mean, prune on vs off):

| scale | prune on | prune off | delta |
|---|---|---|---|
| 10k | 1.05 ms | 1.08 ms | −3% (noise) |
| 50k | 1.90 ms | 1.89 ms | neutral |
| 100k | **2.93 ms** | **2.71 ms** | **+8% worse with pruning** |

## Decision

The reviewer's condition for keeping Task 127 was "demonstrate a config where
the bound activates AND wins." It activates (97.7–98.3% of candidates pruned)
and still loses at the largest scale: the per-lane suffix-bound checks and the
kept-lane bookkeeping cost more than the arithmetic they skip — the LUT
streaming kernel is cheaper run dense than interrupted. The bounded **batch**
scorer is therefore removed (NEON `score_octets_neon_with_min_bound{,_impl}`,
`score_block32_neon_with_min_bound`, `update_live_lanes`,
`BOUND_CHECK_DIM_STRIDE`, the lut32 `*_with_min_bound` batch entry points, the
candidate-batch `..._with_min_bound_for` layer, the IVF
`..._batch_from_payloads_with_min_bound` wrapper, the scan bounded branches,
and the scratch `kept` plumbing).

Kept (unchanged): the per-candidate min-bound path (Task 113,
`score_ip_from_parts_with_min_bound` + `posting_bound_prune` GUC) and
`PreparedLutNoQjl4BitQuery::suffix_max`, which that path consumes.
