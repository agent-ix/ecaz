---
task: 125
topic: bounded-pruner-decision
requester: codex
date: 2026-07-02
code_commit: 10c11bcb0
base_commit: 23f3c752c
---

# Review Request: Task 127 bounded batch scorer — measured in its activating config, then dropped

The task-125/002 closeout review flagged the Task 127 NEON bounded scorer as
dead in the measured config (0/0 dispatch at nprobe=32) and set the condition:
**drop it, or demonstrate a config where the bound activates AND wins.**

## What this packet does

1. **Measures the activating config** (`rerank=heap_f32, rerank_width=100`,
   the shape the task-125 packet used for its prune-fraction evidence) as a
   proper prune-on/off A/B at 10k/50k/100k
   (`task127-pruner-ab-suite.json`, fresh isolated prefixes, provenance-stamped
   manifests).
2. **Result: activates, does not win.** 97.7–98.3% of candidates pruned at
   exact recall parity (1.0000/0.9766/0.9219 both modes) — and latency is
   neutral at 10k/50k and **8% worse with pruning at 100k** (2.93 vs 2.71 ms).
   The per-lane bound checks and kept-lane bookkeeping cost more than the
   arithmetic they skip; the LUT-streaming kernel is cheaper run dense.
3. **Drops the bounded batch scorer** (`10c11bcb0`, −725 lines, net −4 unsafe
   on the scorer surface). The per-candidate min-bound path (Task 113) and
   `suffix_max` it consumes are untouched; `ec_ivf.posting_bound_prune`
   remains functional for the per-candidate paths.

Numbers, artifact paths, and the full removal inventory: `artifacts/manifest.md`.

Pre-existing issues surfaced while validating (out of scope, flagged for
whoever owns them): `tiled_lut_query_prep_rejects_qjl_active_lane` (should-
panic) fails on the pre-change tree; broad `--lib quant` filters hit global-
counter test interference (pre-change tree fails 5 tests, this tree 2).

## Requested review

- Confirm the A/B satisfies the "activates AND wins" condition in the negative
  and the removal scope is right (batch bounded only; Task 113 per-candidate
  path retained).
- With this, Task 127 can be recorded as tried-and-shelved with decision-grade
  evidence, closing the 125-129 closeout's last negative.
