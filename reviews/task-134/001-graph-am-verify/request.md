---
task: 134
topic: graph-am-verify
requester: codex
date: 2026-07-02
code_commit: f248b47fd
base_commit: f248b47fd
---

# Review Request: Task 134 — graph-AM small-batch scoring: verified, negative; cross-AM validation closed

Task 134 asked to investigate a small-batch/graph-AM scorer variant, batching
across frontiers, or a source-grounded decision that the shared partial path is
already adequate — with HNSW/DiskANN evidence at 10k/50k/100k.

## What this packet shows (no code change; measurement + decision)

1. **The premise was stale.** Re-measured at HEAD with per-surface dispatch
   counters: HNSW makes zero candidate-batch flushes in both `exact` and
   `full_lut` modes (recall-identical, latency-indistinguishable), and
   DiskANN's default prefilter batches through the **binary** kernel at
   trivial cost (0.2–0.5 ms kernel per full sweep point). The no-QJL 4-bit
   shared kernel carries no graph-AM traffic in shipping configs on this
   fixture. The 0/39 dispatch claim came from a test-only TQ-prefilter shape
   and predates the task-132 alloc-free driver, which already improved the
   kernel 6–35% at exactly the sub-block widths graph AMs would use.
2. **Decision: source-grounded negative** (the task's third scope option) —
   no small-batch prototype; it would optimize a path with no measured
   traffic. Full reasoning in `artifacts/manifest.md`.
3. **Cross-AM validation (task-125/002 owed flag) is closed on the Apple
   lane**: recall@10 at 10k/50k/100k on the shared-kernel build for
   HNSW (0.9672/0.9479/0.9187), DiskANN (0.9953/0.9896/0.9875), SPIRE
   (1.0000/0.9917/0.9375), IVF (0.9734/0.9521/0.8969) — no anomaly.

All suites provenance-stamped (backend git sha `f248b47fd`); bespoke configs
justified in the manifest (standard lane configs carry no counter flags).

## Requested review

- Confirm the negative satisfies Task 134's gate ("source-grounded negative
  recording why small-batch scoring does not transfer").
- Confirm the cross-AM matrix closes the task-125/002 shared-kernel
  validation flag for the Apple lane (Graviton recorded open, no AWS access).
