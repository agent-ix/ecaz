# Review request: Task 143 — promotion matrix and the two default decisions

- Task: `plan/tasks/143-ivf-tq-1m-promotion-matrix.md`
- Branch: `task-141-sdot-kernel` at `e6b08f497` (single binary, all axes
  GUC/reloption)
- Evidence: `artifacts/manifest.md` (23/23 steps; 100k + 1m-tier, full
  nprobe 8–64 recall sweeps, latency at 32/40, storage, stage counters)
- 1m benches m5-local were operator-authorized 2026-07-02; Graviton is
  explicitly out of scope for now (operator, 2026-07-03).

## Headline (old default → proposed default, nprobe 32)

| scale | latency mean | recall@10 | index size |
|---|---|---|---|
| 100k | 3.16 → **1.75 ms** (−44.6%) | 0.8969 → 0.8938 (−0.31 pp, in noise) | 90.4 → 81.7 MiB (−9.6%) |
| 1m | 12.1 → **7.76 ms** (−35.9%) | 0.9250 → 0.9208 (−0.42 pp, in noise) | 870.6 → 784.8 MiB (−9.9%) |

Or spend it on recall instead: dense-int8 at nprobe 40 beats the old
default on BOTH axes at once — 100k: 2.03 ms @ 0.9031 (vs 3.16 @ 0.8969);
1m: 8.88 ms @ 0.9250 (vs 12.1 @ 0.9208).

## Recommendation 1: flip `ec_ivf.turboquant_scorer` default → `int8_approx`

- Latency −30.0/−32.5% at 1m (row/dense), −33/−34% at 100k, monotone
  across the whole nprobe grid.
- Recall dip ≤0.31 pp (100k) / ≤0.42 pp (1m) at every sweep point —
  within sample noise, consistent with Tasks 136/141.
- HNSW is unaffected (separate GUC, unchanged default; Task 141 proved
  the shared kernel at parity there).

## Recommendation 2: promote `dense_posting_blocks=1` for TurboQuant ec_ivf

- Recall byte-identical to row at EVERY nprobe point, both scales, both
  scorers (32 recall cells, zero divergence).
- Storage −9.6/−9.9%.
- Latency: −5.0% (1m lut) to −16.1% (100k lut) mean, and the win grows
  under buffer pressure exactly as the Task 135 analysis predicted —
  which is the regime a real 1m+ deployment lives in.
- Insert/mixed-boundary behavior proven in the Task 142 packet
  (three-way parity on a post-insert mixed table).
- This satisfies the 1m-evidence prerequisite the Task 111a closeout set
  for reopening promotion. Scope: TurboQuant `ec_ivf` builds (the RaBitQ
  dense default question stays with the 111a family).

## Caveats

- m5-local lane only; Graviton remains the recorded cross-lane follow-up
  (operator deferred).
- "1m" = the 990k anchor split (the maximal held-out-query fixture
  derivable from the 1M release).
- Recall samples are 32 (100k) / 24 (1m) queries — noise bands are wide;
  the byte-identity claims (dense vs row) are exact regardless.
- The 100k cells ran under 1m-induced buffer pressure (documented in the
  manifest); within-run comparisons are unaffected.

## Asks

1. Approve both default flips. On approval I will land them as one
   narrow commit (GUC default + reloption default + doc lines + the two
   unit tests that pin session defaults) plus a confirming
   default-path A/B cell (fresh no-reloption, no-GUC load vs this
   packet's dense-int8 cells).
2. Confirm the anchor-split framing for the 1m tier is acceptable
   provenance for the promotion decision.
