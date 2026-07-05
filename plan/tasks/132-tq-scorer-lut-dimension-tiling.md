# Task 132: TQ scorer LUT dimension tiling (durable L1D fix)

Status: **proposed** (2026-07-01). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 125.

## Why

Task 125 proved the no-QJL 4-bit scorer is **L1D-residency bound**, not
arithmetic- or bandwidth-bound: `BLOCK_WIDTH 32→64` (fewer LUT reloads) gave
nothing, while f32→i16 (smaller LUT working set) gave ~2×. The f32 LUT is 98 KB
and i16 is 49 KB; L1D is **64 KB on every Graviton (2→5)** and 128 KB on Apple.
So the i16 win is a lower bound on Graviton (f32 hard-spills there), and the LUT
still does not comfortably fit 64 KB L1D as dimensions grow.

Dimension tiling caps the LUT working set **independent of dim**: score a tile
of dimensions (whose LUT sub-block fits L1D) across all candidates in a block,
then advance to the next tile. This is the original Task 125 "cache-block over
dimensions" scope that was left unfulfilled when int16 compaction was chosen
instead.

## Scope

- Tile `score_lut_no_qjl_4bit_*` over dimension ranges so the per-tile LUT
  sub-block × block-width working set stays under a target L1D budget.
- Keep the int16 LUT representation (Task 125) — tiling composes with it.
- Sweep tile size against L1D budgets (64 KB Graviton, 128 KB Apple).
- Preserve octet/tail correctness and bit-consistency with the scalar reference.

## Out of Scope (hard)

- No new on-disk rerank format/mode/reloption. Optimizes the existing 4-bit
  no-QJL scorer only.

## Required Evidence

- Multi-host A/B: `ecaz bench suite` IVF recall+latency+storage at 10k/50k/100k
  on Apple, Graviton 4, and Intel. The load-bearing result is **Graviton**
  (64 KB L1D), where tiling should help most.
- Gated on Task 125 packet 001's i8 A/B outcome: if i8 already sits at the L1
  floor with no gain, tiling likely also yields little on that host — record the
  cache-bound-vs-floor evidence either way.

## Gate

- A measurable latency/kernel improvement at unchanged recall and storage on at
  least the Graviton lane, OR a source-grounded "already at L1 floor" negative
  with per-host cache evidence.

## Exit Criteria

- Tiled scorer lands behind the existing shared kernel with bit-consistency
  tests across dims and tile sizes, or is shelved with multi-host evidence.
- No new unsafe fn; no anti-pattern B.
