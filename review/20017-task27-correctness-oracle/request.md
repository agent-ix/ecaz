# Review Request: Task 27 Slice 1 — Symphony Correctness Oracle

Scope: documentation only. Freezes the Phase-0 reference oracle that
Stage 2 must satisfy before quantization-aware pruning or padded
adjacency are allowed to change behavior.

Task: `plan/tasks/27-symphony-access-method.md` Phase 0
("Correctness invariant").

Branch: `task27-symphony-stage2-phase0-oracle` (slice 1 builds on
`09bce2c`, the task-25 fast-forward now on `main`).

Read set used for this packet:
- `plan/tasks/27-symphony-access-method.md`
- `review/20015-task25-task27-handoff-contract-v2/request.md`
- `spec/adr/ADR-045-symphonyqg-quantized-graph-access-method.md`
- `spec/adr/ADR-041-module-structure-for-multi-am-multi-quantizer-growth.md`
- `~/dev_bak/papers/symphonyqg-2025-sigmod-arxiv-2411.12229.pdf`

## Goal

Stage 2 introduces two structural changes and one query-path change:

1. centered per-neighbor RaBitQ codes
2. multi-visit beam search over those codes
3. later, quantization-aware pruning + padding

If all three move at once, any regression is ambiguous. This packet
locks the first equivalence target so the implementation can prove the
new page codec and centered scan mechanics independently of the new
graph-construction rules.

## Oracle definition

The Phase-0 oracle is:

> A `symphony` index built with `padding = 1` and **fp32-scored
> pruning** must return identical top-k to an equivalent
> `ec_hnsw`-shape reference that runs the **same centered-RaBitQ
> scan** over the same graph.

Concretely, both sides must share all of the following:

- same corpus
- same insertion / build order
- same `m`, `ef_construction`, and entry-point selection
- same graph edges, produced by fp32 neighbor selection
- same query set
- same centered RaBitQ scorer inputs:
  - `RaBitQQuantizer::prepare_center`
  - `RaBitQQuantizer::encode_code_centered`
  - `RaBitQQuantizer::prepare_scorer_centered`
  - `CenteredScorer::score_at`
  - `RaBitQQuantizer::centered_residual_magnitude`
- same beam width and same visit bookkeeping
- same exact rerank tail for the final top-k

The only permitted difference is the storage / decoding surface:

- `symphony` side reads centered residual codes from the new padded-
  adjacency page codec
- reference side reads equivalent adjacency and centered-code state
  through a test helper layered on the existing `ec_hnsw` graph shape

## Why the oracle uses fp32 pruning and padding = 1

This oracle isolates **page layout + centered scan correctness**.

- `padding = 1` removes the out-degree-refinement variable.
- fp32-scored pruning removes the quantization-aware build variable.
- exact rerank stays on, so Stage 3's no-rerank risk is not in scope.

If the two systems disagree under this setup, the bug is in one of:

- centered-code packing / unpacking
- neighbor tuple decoding
- per-visit center handling
- beam bookkeeping, including multi-visit semantics
- rerank candidate handoff

It is **not** evidence against Symphony's structural changes
themselves, because those are held constant here.

## Reference harness shape

The reference is not a user-facing AM. It is a test-only oracle that
reuses the existing `ec_hnsw` graph topology and adds the Symphony
scoring rules around it.

Expected harness layers:

1. Build a graph with current fp32 selection logic.
2. For every vertex/neighbor edge, derive the centered code via the
   task-25 API and store it in a helper-owned structure.
3. Run a Symphony-style beam search over that helper structure:
   - centered scores during traversal
   - same multi-visit allowance as the real `symphony` scan path
   - exact rerank for final top-k
4. Compare that transcript against the real `symphony` AM using
   `padding = 1`.

The reference harness is intentionally narrower than a production AM:
it exists to tell us whether the new on-disk adjacency codec preserves
the same search semantics.

## Exact equality requirement

The assertion is **identical top-k heap TID order**, not merely recall
parity.

Stronger optional checks are encouraged once helper seams exist:

- identical visited-vertex transcript
- identical rerank candidate pool before exact sort
- identical traversal scores within a small f32 tolerance

But the required gate for this slice is top-k identity.

## What this oracle does not prove

- It does not validate quantization-aware pruning.
- It does not validate out-degree padding.
- It does not validate Stage 3 no-rerank.
- It does not validate the SIMD hot loop; task 25 already gives the
  scalar correctness reference.

Those land as separate gates later so failures remain attributable.

## Immediate consequences for implementation order

Phase 1 and early Stage 2 should land in this order:

1. page codec that can carry centered per-neighbor codes
2. centered scan path over that codec
3. oracle test proving top-k identity at `padding = 1`
4. only then quantization-aware pruning
5. only then padding / refinement

This is the narrowest path that keeps regressions debuggable.

## Closing

This packet freezes the first non-negotiable correctness target for
task 27: before Symphony is allowed to change the graph, it must prove
that its new adjacency codec and centered scan path reproduce the same
answers as an equivalent `ec_hnsw`-shape reference on the same graph.
