# Task 134 Graph-AM Verify-First + Cross-AM Validation Artifacts

- task bucket: `reviews/task-134/001-graph-am-verify`
- measurement commit: `f248b47fd` (installed backend `ecaz_build_git_sha() =
  f248b47fd…`, i.e. int16 LUT + alloc-free driver + Task-127 bounded-scorer
  removal; recorded in both suite manifests' backend blocks)
- lane: local PG18 (Homebrew 18.3), Apple M5 Pro
- fixture: staged real corpus (dbpedia 1536-dim; SHAs in task-133 packet
  manifest), bits=4, seed=42, k=10; fresh isolated
  `task134_{hnsw,diskann,spire}_real{10k,50k,100k}` prefixes
- suite configs (bespoke: standard configs carry no counters flags, and the
  verify-first question needs the width-histogram/dispatch counters):
  - `task134-graph-am-suite.json` — HNSW (m=16, ef_construction=128) +
    DiskANN: load/recall/latency/storage; SPIRE: load/recall. Default
    registered sweeps (hnsw [40..200], diskann [64..800], spire [8..32]),
    `task87_candidate_batch_counters` on latency steps.
  - `task134-graph-am-fulllut-suite.json` — HNSW recall+latency re-run under
    `ec_hnsw.turboquant_exact_score_mode=full_lut` (reuses the same indexes).
- results: `results.jsonl`, `results-fulllut.jsonl` + per-step logs

## Verify-first findings (the task's premise, re-measured at HEAD)

1. **HNSW performs ZERO candidate-batch flushes into any shared kernel on this
   fixture** — `surface=hnsw flushes=0` at every scale/ef under BOTH the
   default `exact` (QJL) mode and the `full_lut` GUC; no
   `[block-kernel-counters]` rows at all in the HNSW latency logs. Exact-mode
   TQ scoring flows through the per-candidate score-and-cache path. FullLut vs
   Exact is also **recall-identical at every ef and scale** and
   latency-indistinguishable (e.g. 100k ef=200: 2.07 vs 2.10 ms).
2. **DiskANN's default prefilter batches through the BINARY kernel, not the
   no-QJL 4-bit LUT kernel**: `surface=diskann quant=binary`, frontier-sized
   flushes (width histogram dominated by <8/8–15/16–31), total kernel time
   0.22–0.47 ms per whole sweep point (~45–98k candidates) — a trivial share
   of 0.85–4.13 ms queries.
3. The "DiskANN dispatch flipped 32/7 → 0/39 kernel/scalar" claim from the
   task-125 review is **stale twice over**: it described a test-only
   TurboQuant-prefilter configuration, and the alloc-free driver (task-132
   packet) has since replaced the dispatch entirely — all sub-block widths now
   take the octet-granular NEON path with measured microbench wins at exactly
   graph-AM widths (w8 −35%, w16 −22%, w24 −19%, w39 −6%).

## Cross-AM recall matrix (the task-125/002 owed validation)

recall@10 on the shared-kernel build (`f248b47fd`), best sweep point:

| AM | 10k | 50k | 100k |
|---|---|---|---|
| ec_hnsw (ef=200) | 0.9672 | 0.9479 | 0.9187 |
| ec_hnsw full_lut (ef=200) | 0.9672 | 0.9479 | 0.9187 |
| ec_diskann (ls=800) | 0.9953 | 0.9896 | 0.9875 |
| ec_spire (nprobe=32) | 1.0000 | 0.9917 | 0.9375 |
| ec_ivf (nprobe=32, task-133 packet) | 0.9734 | 0.9521 | 0.8969 |

No recall anomaly on any AM; the SPIRE/HNSW/DiskANN gap flagged in the
task-125 closeout is closed on the Apple lane.

Latency summaries: `results.jsonl` (e.g. DiskANN 100k: 0.85/1.17/1.48/2.46/4.13
ms at list_size 64/128/200/400/800; HNSW 100k: 0.79→2.07 ms across ef 40→200).
Storage per AM in `storage-*.log`.

## Decision (task gate)

Source-grounded negative, third option of the task's Scope: **the shared
partial/small-batch path is already adequate for graph AMs.** In shipping
configurations the no-QJL 4-bit kernel is not on either graph AM's hot path
(HNSW: per-candidate cache path, mode-neutral; DiskANN: binary prefilter
kernel at trivial cost), and the small-batch widths those AMs would use are
exactly where the task-132 alloc-free driver already improved the kernel
6–35%. Building a dedicated small-batch scorer or cross-frontier batching
would optimize a path that carries no measured traffic — no prototype is
justified. Graviton lane remains open as with the sibling tasks (no AWS
access this session).
