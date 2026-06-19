# Task 111g: IVF Coarse-Rerank Representations (generic rerank_format)

Status: **proposed**.
Priority: P1 latency / recall-at-latency (unlocks the 111e contract).
Parent: `111-ivf-scan-dense-posting-block-layout.md`, follows
`111e-ivf-coarse-rerank-candidate-pipeline.md` and `111f-ivf-dead-format-cleanup.md`.
Evidence anchor: `reviews/task-111e/005-compact-sidecar-rerank/`,
`reviews/task-111e/006-final-gate/feedback/2026-06-18-02-reviewer.md`.

## Goal

Make IVF rerank **pluggable by `rerank_format`**, so the coarse_rerank survivor
set can be reranked with representations other than heap-f32 — reusing the
existing `candidate_batch` quant scorers rather than new SIMD. Realize the
packet-005 wins: table-side **f16** (matches f32 recall at half bytes) and
**rabitq4**, read **tid-sorted**.

Today rerank is hardcoded: `rerank_probe_candidates` dispatches on `RerankMode`
(`heap_f32` / `source_column` only); the `rerank_format` enum
(f32/heap_f32/rabitq2/4/8/turboquant) is parsed and stored but **no code path
consumes it**, and the compact values + `rerank_placement = index` are rejected
at index creation. This task wires `rerank_format` through to an actual rerank
scoring path.

## Why

111e proved coarse_rerank's value but shipped heap-f32 only; 005 measured that
f16 preserves f32 recall at half the bytes and that table-side tid-sorted reads
are the dominant IO lever. The scoring kernels already exist
(`candidate_batch::score_rabitq_bitsn_batch_for`, etc.), so this is mostly
**plumbing**: a rerank dispatch keyed on `rerank_format` + payload fetch +
scoring via the shared codec surface. This unlocks the flexibility the 111e
contract advertised.

## Scope

- IVF `coarse_rerank` only; build on the post-111f keeper dense path.
- **Generic rerank dispatch:** refactor the rerank stage to select the rerank
  scorer by `rerank_format`, not just `RerankMode`. Keep `heap_f32` as the f32
  path with **bit-identical results** to today (no regression for the existing
  mode).
- **Table-side compact reps:** implement `f16` and `rabitq4` table-side
  (sidecar) rerank, reading the rerank payload **tid-sorted** (per 005). Add
  `f16` to the `rerank_format` enum (005 gap). Lift the rejection for each
  format as it lands; keep rejecting the unimplemented ones.
- **Reuse the shared scorers:** rerank scoring goes through the existing
  `candidate_batch` codec scorers — no new SIMD kernels.
- Counters/admin: rerank rep + placement visible in EXPLAIN / admin snapshot.

## Non-Goals

- **Index-side rerank placement** (`rerank_placement = 'index'`) — keep rejected;
  scope it as a follow-on slice/task if it proves large (note it in the closeout).
- TurboQuant / rabitq2 / rabitq8 rerank unless they fall out for free (005
  rejected rabitq8 for the high-recall path; rabitq2 likely too lossy).
- Changing the coarse stage, posting layout, quant math, or recall semantics.
- Lazy-rerank fetch reduction — that's Task 112; coordinate, don't duplicate.

## Phases

1. **Generic dispatch refactor.** Route the rerank stage through a
   `rerank_format`-keyed path; prove `heap_f32` output is bit-identical to the
   pre-refactor path (equivalence test).
2. **Table-side f16 + rabitq4.** Implement both via the shared scorers,
   tid-sorted reads; add `f16` to the enum; lift their rejections; PG18 fixtures
   asserting recall parity (f16 ≈ f32) and the admin snapshot.
3. **Benchmark gate.** Per the 006 follow-up: 50k/100k, coarse_rerank with
   rerank_format ∈ {f32, f16, rabitq4}; recall / latency / storage; **plus the
   matched-recall baseline** (coarse_rerank vs dense-rb8 / row-f32 at recall
   ≈0.97 and ≈0.99). Explicit promote/iterate recommendation.

## Acceptance Criteria

1. Rerank stage dispatches on `rerank_format`; `heap_f32` results are
   bit-identical to the pre-refactor path (equivalence test).
2. `f16` and `rabitq4` table-side rerank implemented and creatable via SQL
   (rejections lifted; `f16` in the enum); admin snapshot reports them.
3. Rerank scoring reuses the `candidate_batch` codec scorers (no new SIMD).
4. Table-side reads are tid-sorted.
5. PG18 fixtures: recall parity (f16 ≈ f32) + correctness for rabitq4.
6. A benchmark packet reports the rerank-rep matrix + matched-recall baseline at
   50k/100k with an explicit promote/iterate decision.

## Dependencies and Coordination

- Builds on 111e (`coarse_rerank`) + 111f (keeper dense path) on the 111 lane.
- Reuses the Task 87 `candidate_batch` / `QuantCodecKind` shared scorer surface.
- Coordinates with Task 112 (lazy heap-f32 rerank): 112 reduces rerank *fetch*
  cost; 111g adds rerank *representations*. Keep the fetch path factored so both
  compose.
- This is the last 111-lane slice before the keepers + coarse_rerank + 111g merge
  to `main`; 112/113/115 branch off the merged main.
