# Review Request: A4 Quantizer Triage

Basis: `main` at `2d14bea` plus local quantizer-ablation probes in `tests/recall_integration.rs`

## Summary

- this packet is a follow-up to `200-a4-recall-gate-rerun`
- its purpose is to answer a narrower question: after the repaired `10k` A4 rerun, is the main
  blocker still graph traversal, or is the exact quantized path itself too weak against fp32 truth?
- the new exact-only evidence shifts suspicion away from a graph-only explanation and toward the
  quantized path itself

## Reference Hypotheses

These are the stable A4 labels to use:

1. `H1: graph-runtime gap`
2. `H2: quantized-objective mismatch`
3. `H3: quantized-path implementation defect`

## New Evidence

### 1. Pure-Rust exact-only bitwidth sweep on uniform `1k x 1536`

This uses the existing `tests/recall_integration.rs` harness and removes HNSW from the equation.

- `2-bit`: Recall@10 `32.0%`
- `3-bit`: Recall@10 `49.0%`
- `4-bit`: Recall@10 `49.5%`
- `5-bit`: Recall@10 `51.5%`
- `6-bit`: Recall@10 `51.0%`
- `7-bit`: Recall@10 `52.0%`
- `8-bit`: Recall@10 `52.0%`

Takeaway:
- the exact quantized path is already weak on this workload
- the problem is not specific to graph traversal
- increasing bitwidth from `4` to `8` barely helps in this harness

### 1a. 1536 padding ablations on uniform `1k x 1536`, `4-bit`

These probes isolate the `1536 -> 2048` rotation mismatch described in `ADR-021`.

- `current_exact`: Recall@10 `49.5%`
- `transform_cb_exact`: `50.5%`
- `tail_ref_cb1536`: `82.0%`
- `tail_ref_cb2048`: `80.5%`
- `tail_full_cb1536`: `83.0%`
- `tail_full_cb2048`: `81.0%`

Takeaway:
- changing only the analytic codebook prior from `d=1536` to `d=2048` is a small move
- keeping the full rotated tail is a huge move
- the dominant loss is not “wrong Beta prior”; it is discarding the `[1536, 2048)` rotated tail
- adding the full-tail QJL term only nudges the full-tail reference slightly above the full-tail
  MSE-only baseline, so QJL remains secondary here

### 2. Pure-Rust exact-only clustered sweeps

Clustered `1k x 1536`:

- `2-bit`: Recall@10 `32.0%`
- `3-bit`: Recall@10 `45.5%`
- `4-bit`: Recall@10 `55.5%`
- `6-bit`: Recall@10 `57.0%`
- `8-bit`: Recall@10 `59.0%`

Clustered `10k x 1536`, `4-bit`:

- Recall@10 `41.0%`

Takeaway:
- even on clustered data, the exact quantized path stays far below the intended A4 gate
- the repaired SQL `10k` exact result (`43.1%`) is consistent with these pure-Rust exact-only runs

### 2a. 1536 padding ablations on clustered `1k x 1536`, `4-bit`

- `current_exact`: Recall@10 `55.5%`
- `transform_cb_exact`: `53.5%`
- `tail_ref_cb1536`: `80.5%`
- `tail_ref_cb2048`: `80.5%`

Takeaway:
- the same pattern holds on a more structured corpus
- codebook-dimension substitution is not a reliable win and should be recorded as a failed primary
  fix direction
- retaining the rotated tail again produces the large quality jump

### 3. Near-duplicate preservation at `1536-dim, 4-bit`

For self vs a tiny perturbation:

- angle `0.01`: `53.0%` preserved
- angle `0.02`: `57.0%`
- angle `0.05`: `64.0%`
- angle `0.10`: `81.0%`
- angle `0.20`: `100.0%`

Takeaway:
- the quantized scorer does not reliably preserve very small ranking differences
- that again points upstream of the graph path

### 4. Scorer ablation on uniform `1k x 1536`, `4-bit`

Exact-only comparison on the same corpus:

- `exact`: Recall@10 `49.5%`
- `gamma_zero`: `49.0%`
- `code_proxy`: `47.5%`
- `decoded`: `49.0%`

Takeaway:
- `gamma` is not the main reason the exact path is weak
- the full scorer is only marginally better than a much simpler proxy
- brute-force search over decoded approximate vectors is no better than the current exact scorer

## What This Rules Down

- `H1` cannot be the whole problem.
  - graph recall is below exact at the required budgets, so there is still runtime work to do
  - but exact quantized recall is already too low for a graph-only explanation
- `196` neighbor-slot packing was a real runtime/storage bug and was worth fixing, but it does not
  explain the remaining exact-only ceiling
- `195` and `197` look secondary, not primary
  - score/gamma ablations move the exact-only result only slightly
- “use `transform_dim` in `lloyd_max`” also looks secondary
  - it is worth recording as attempted, but it does not explain the missing `30+` Recall@10 points
- `198` can only explain graph-vs-exact behavior, not exact-vs-fp32 behavior

## Current Read

The dominant live suspicion is now one of:

1. `H2: quantized-objective mismatch`
   - the current `tqvector` objective may simply not track fp32 top-k closely enough for the A4
     gate on this corpus

2. `H3: quantized-path implementation defect`
   - the quantized path may contain a deeper flaw, but if so it is likely in the encoding/storage
     formulation itself rather than in graph traversal

The strongest concrete sub-hypothesis now is:

- `H3a: transform-tail truncation is the dominant quality loss at 1536`
  - `ADR-007` explicitly chose to persist only the first `d` transform coordinates
  - `ADR-021` already calls out that 1536-dim inputs are rotated in 2048-space and discard 25% of
    the transformed signal energy
  - the new ablations are consistent with that exact mechanism

The code surfaces that currently look most relevant are:

- `src/quant/prod.rs`
- `src/quant/mse.rs`
- `src/quant/codebook.rs`
- `spec/adr/ADR-007-query-scoring-and-payload.md`
- `spec/adr/ADR-021-default-vector-dimension.md`

The notable structural fact is that the current path is using:

- scalar per-coordinate quantization in rotated space
- a data-free analytic codebook
- a lightweight residual/QJL correction

That may simply be too weak a representation for the stated recall target, but this packet does not
claim that yet. It only narrows where the remaining cause most likely lives.

## Suggested Next Checks

1. confirm whether the quantizer formulation is intended to support high fp32 top-k recall at this
   target, or whether the current design is knowingly approximate in a way the A4 gate does not
   account for
2. evaluate a product-compatible mitigation for 1536 that avoids transform-tail loss:
   - tiled FWHT for the 1536 compatibility tier
   - or a tail-retaining storage/reference variant if the storage hit is acceptable
3. audit `prod.rs` end-to-end for any remaining implementation defect, but treat codebook-dimension
   substitution as a ruled-down primary fix rather than the default next move

## Review Focus

- whether the new exact-only results fairly shift the center of gravity from `H1` toward `H2/H3`
- whether the scorer ablation is enough to treat `195/197` as secondary for now
- whether the next A4 slice should stay in graph runtime or move into the quantizer formulation
