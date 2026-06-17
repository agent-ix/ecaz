# Task 111d: IVF Pre-Transposed Canonical Block Geometry

Status: **proposed**.
Priority: P1 latency (incremental layer on score-in-place).
Parent: `111-ivf-scan-dense-posting-block-layout.md`.
Depends on: **`111c-ivf-page-aware-scatter-scorer.md`** (and 111b).
Evidence anchor: `reviews/task-111a/{004,007,008}`.

## Goal

Skip the per-query, per-posting transpose entirely by storing the Task 111b
columnar payload **pre-transposed at build time** into a **canonical,
cross-ISA block geometry**, so the scan does at most a light per-ISA adapt
instead of a full row→dim-major transpose. The build pays the transpose once;
every query benefits.

## Why

Even after Task 111c removes the assembly copy, each scan still transposes
row-major payloads into per-dim columns (`transpose_8x16` and siblings) on every
query. Frozen lists are immutable, so that transpose is redundant work repeated
across all queries — it can be done once at build.

The catch is portability: a transposed layout tiled for one ISA's registers
(AVX2 256-bit) is wrong for another (NEON / SVE2-128), which would force an index
rebuild on architecture change and break Intel↔Graviton volume/snapshot moves.
This task therefore stores a **canonical** geometry — neutral dim-major width-32
blocks — that every target ISA consumes with only a light load-time adapt
(stride/permute), not a full transpose. Same on-disk bytes on Intel and
Graviton, no rebuild; the only residual cost is the small adapt, which is far
less than today's full transpose.

## Scope

- A canonical dim-major block layout (target width 32) for the 111b payload
  column, chosen so AVX2, SVE2-128, and NEON each adapt it with minimal
  permute/strided-load work (no full transpose at scan).
- Build-time transpose into the canonical layout; deterministic; gated tag or a
  layout flag in the 111b list header.
- Per-ISA light-adapt readers feeding the 111c scatter scorer (which now skips
  the transpose stage).
- Volume portability preserved: identical bytes across little-endian targets; an
  index built on one arch scans correctly on the other with no rebuild.
- Equivalence tests vs the 111c (scan-time-transpose) path — identical scores.

## Non-Goals

- The columnar format (111b) and the scatter scorer (111c).
- **Host-pinned ISA-exact tiling** (the maximal-throughput, non-portable
  variant) — that is the deferred future **Task 111e** escape hatch, opt-in for
  arch-stable deployments; not in scope here.
- Changing scoring math / recall / quantization.

## Phases

1. Choose + specify the canonical width-32 dim-major geometry; prove each target
   ISA adapts it with a light, documented shuffle.
2. Build-time transpose writer + decode; equivalence tests vs 111c.
3. Per-ISA adapt readers wired into the 111c scatter scorer (transpose skipped).
4. Benchmark gate: latency vs 111c (scan-time transpose) across TQ + RaBitQ
   {1,2,4,8} at 50k/100k; confirm recall parity, the residual-adapt cost, and
   cross-arch portability (build on one lane, scan on the other, no rebuild).

## Acceptance Criteria

1. Canonical pre-transposed layout implemented behind a gate; deterministic
   build; same bytes scan correctly on AVX2, SVE2-128, and NEON with no rebuild.
2. Scores bit-identical to the 111c scan-time-transpose path.
3. Recall and NDCG unchanged across the matrix.
4. A benchmark packet quantifies the latency gain vs 111c and the residual
   per-ISA adapt cost, and confirms Intel↔Graviton volume portability (no
   rebuild on arch change).
5. The packet states whether the pre-transpose gain justifies the format, and
   notes the deferred host-pinned (111e) escape hatch as the remaining lever for
   arch-stable deployments.

## Dependencies and Coordination

- Hard dependency on Task 111c (scatter scorer) and 111b (format).
- ADR-077 per-ISA block-kernel coverage governs the adapt-reader gates.
- Task 42: a new layout flag/tag in the columnar header — reconcile.
- Future **Task 111e** (host-pinned compaction): opt-in rewrite to local ISA
  tiling for arch-stable deployments; deferred by operator decision (2026-06-17)
  — create only when prioritized.
