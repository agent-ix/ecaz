---
task: 132
topic: alloc-free-batch-driver
requester: codex
date: 2026-07-02
code_commit: c5201bffc
base_commit: a05babf74
---

# Review Request: Task 132 — alloc-free batch driver lands; dimension tiling shelved on evidence

Task 132 asked for LUT dimension tiling as a durable L1D fix, gated on either a
measurable win (Graviton being the load-bearing host) or a source-grounded
"already at L1 floor" negative with per-host cache evidence.

## What landed

- `87eb5ad13` — env-gated release width microprofile for
  `score_lut_no_qjl_4bit_batch_tiled` (committed before the driver change so
  before/after numbers bracket it).
- `c5201bffc` — the reviewer-blocked hot-path allocations are gone:
  `score_lut_no_qjl_4bit_batch_tiled` now drives full BLOCK_WIDTH slices through
  the existing stack-scratch octet kernel and hands the entire sub-block
  remainder to the partial path in one octet-granular call.
  `score_batch_tiled_neon{,_impl}` (per-call `vec![]` accumulators, per-chunk
  `vec![]` transpose columns) are deleted, as are the non-multiple-of-8 padding
  vecs in the dispatch. Net −93 lines, −2 `unsafe` blocks. Bit-exactness suite
  12/12 (dims × widths matrix unchanged).

## Evidence (artifacts/manifest.md is source of truth)

- Microbench: 6–35% faster ns/candidate at widths 8–64 (the graph-AM regime)
  and 9–11% at widths 256–1024 (IVF slab regime); no width regresses.
- e2e IVF A/B (slice-3 dylib vs same-session baseline): recall identical at
  10k/50k/100k, kernel neutral within noise, storage unchanged. e2e latency is
  not claimed (fresh table rebuild confounds it).
- 64 KB-L1D probe (QoS-steered to the M5 Pro's Performance cluster): width
  curve flat from 32 up — no residency cliff. Caveats recorded.

## Dim-tiling decision — shelve, with the negative the task's gate allows

The i16 LUT (task 125) is 48 KiB at dim 1536: it fits the 64 KB Graviton-class
L1D for **every dim ≤ 2048**. The premise tiling was scoped for (LUT > L1D) is
gone on all current targets; Apple e2e was neutral twice; the 64 KB-core probe
shows no cliff. Full reasoning + numbers in `artifacts/manifest.md`.

Open item: a Graviton lane run stays recorded as the formal cross-check if a
dim > 2048 fixture becomes a target (AWS unreachable this session — no
credentials).

## Requested review

- Confirm the alloc-free driver is landable as-is (bit-exact, hygiene-positive,
  no e2e regression).
- Confirm the shelve-dim-tiling rationale satisfies Task 132's negative-exit
  clause given the host constraint, or name the additional evidence required.
