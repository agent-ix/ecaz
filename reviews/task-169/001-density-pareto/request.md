# Task 169 packet 001 — coarse-payload density pareto (review request)

Status: **measured — awaiting review**. Coder: Codex. 2026-07-03.
Measurement-only packet (no code change; all cells on the installed
`da6101a00` binary, same as the Task 168 champion cells).

## Summary

Task 169 asked whether a denser coarse payload + exact rerank beats the
newly promoted TQ dense-int8 default. Phase-0 found TQ's coarse encode
hardwired to 4-bit (a TQ 2-bit lane would be new surface — Task 96's
deferred scope), so the hypothesis was tested on existing surfaces:
RaBitQ `quant_bits={1,2}` + `dense_posting_blocks=1` +
`rerank='heap_f32'` (width 50) vs the champion, at 10k/50k/100k and
1m for the winner.

**Headline: rb1 pareto-dominates the current default at every scale.**

| scale | latency n32 (TQ → rb1) | recall n32 (TQ → rb1) | index |
|---|---|---|---|
| 10k | 0.60 → 0.67 ms | 0.975 → **1.000** | — |
| 50k | 1.12 → 1.12 ms | 0.959 → **0.994** | — |
| 100k | 1.61 → **1.58 ms** | 0.894 → **0.938** | 81.7 → **27.6 MiB** |
| 1m | 6.66 → **6.21 ms** | 0.9208 → **0.9667** | 784.8 → **247.8 MiB** |

- rb1 == rb2 recall at all 18 shared points — the rerank-masking fact
  (Task 115/122) reconfirmed head-on; rb2 is 4–5× slower (off the
  popcount kernel path) and strictly dominated → the 2-bit branch of
  the question (and the reframed Task 96) closes as not-worth-building.
- Rerank cost is bounded: ~0.18 ms/query at 100k, ~0.49 ms/query at 1m
  (exact_rerank stage), already included in the latency numbers.
- Alternative read: rb1 at n16 (0.925 @ 1m) matches-or-beats the TQ
  default's n32 recall at roughly two-thirds of its latency.

## Decision asks

1. Accept the pareto verdict and the Task 96 closure (2-bit not worth
   new surface; density pays at 1 bit).
2. Whether to open the **default-format promotion question** (rb1-style
   configuration or the Task 111e `coarse_rerank` format vs
   `turboquant`) as its own task with Task 143-grade promotion
   discipline: cold-cache, insert/churn, second corpus, Graviton, and
   the nprobe operating-point re-derivation. This packet's warm-cache
   m5-local evidence is strong but deliberately NOT a promotion claim.

## Evidence

`artifacts/manifest.md` (cells, shas, full tables, verdict);
`artifacts/cells/` (≤100k, 25/25 steps) + `artifacts/cells-1m/` (4/4
steps): results.jsonl, recall/latency/storage logs with stage counters,
sha prechecks; suite configs `task147-density-pareto-suite.json` +
`task147-rb1-1m-suite.json` (bespoke reason in manifest). The first
≤100k run's config error (`bits` vs `quant_bits` reloption) is
documented in the manifest run log.
