# Review Request: A4 1536 Tail-Truncation Probes

Basis: `main` at `2d14bea` plus local exact-only probes in `tests/recall_integration.rs`

## Summary

- this packet follows `200-a4-recall-gate-rerun` and `201-a4-quantizer-triage`
- its purpose is narrower: determine whether the weak exact ceiling at `1536-dim` is mostly coming
  from the current scoring details, from the analytic codebook prior, or from the decision to keep
  only the first `d` coordinates after a `2048`-dim SRHT
- the new evidence points strongly at the last of those: transform-tail truncation

## Reference Hypotheses

Use the existing A4 labels:

1. `H1: graph-runtime gap`
2. `H2: quantized-objective mismatch`
3. `H3: quantized-path implementation defect`

This packet narrows `H3` further:

- `H3a: 1536 transform-tail truncation is the dominant quality loss`

## Why This Probe Exists

`ADR-007` explicitly chose to persist only the first `d` transform coordinates.

For `d=1536`, `ProdQuantizer` currently does:

1. zero-pad to `2048`
2. apply SRHT in `2048` space
3. quantize and persist only the first `1536` rotated coordinates
4. treat the discarded `[1536, 2048)` tail as zero during reconstruction

`ADR-021` already flags this as a likely quality problem. This packet tests whether that is a small
effect or the main exact-only ceiling.

## New Experiments

### 1. Uniform `1k x 1536`, `4-bit` padding ablations

- `current_exact`: Recall@10 `49.5%`
- `transform_cb_exact`: `50.5%`
- `tail_ref_cb1536`: `82.0%`
- `tail_ref_cb2048`: `80.5%`
- `tail_full_cb1536`: `83.0%`
- `tail_full_cb2048`: `81.0%`

Read:

- swapping only the codebook prior from `1536` to `2048` is a minor move
- retaining the full rotated tail is a massive move
- adding the full-tail QJL term moves the result only slightly above the full-tail MSE-only
  baseline, so QJL remains secondary

### 2. Clustered `1k x 1536`, `4-bit` padding ablations

- `current_exact`: Recall@10 `55.5%`
- `transform_cb_exact`: `53.5%`
- `tail_ref_cb1536`: `80.5%`
- `tail_ref_cb2048`: `80.5%`

Read:

- the same pattern holds on structured synthetic data
- the codebook-only substitution should be recorded as a failed primary fix direction
- the large recovery again comes from keeping the rotated tail

### 3. Clustered `1k x 1536`, `4-bit` tiled-FWHT reference

This uses a block-diagonal `3 x 512` FWHT instead of zero-padding to `2048`, while still storing
only `1536` transformed coordinates.

- `current_exact`: Recall@10 `55.5%`
- `tiled_full_cb512`: `72.5%`
- `tiled_full_cb1536`: `81.5%`

Read:

- a product-compatible mitigation exists in principle for the `1536` compatibility tier
- avoiding the padded `2048` transform tail loss recovers most of the missing exact-only recall
- the strong `cb1536` result suggests the current analytic codebook prior is not obviously the next
  thing to replace before trying a tiled rotation

## Failed Directions Worth Recording

These are useful negative results and should not be lost:

- `transform_cb_exact` on uniform `1k`: only `+1.0` Recall@10 point
- `transform_cb_exact` on clustered `1k`: `-2.0` Recall@10 points
- full-tail QJL adds little beyond full-tail MSE-only
  - `82.0% -> 83.0%` on uniform `1k`

These do not rule out follow-up tuning later, but they do rule them down as the main A4 blocker.

## Updated Read

The dominant explanation is now:

- the current `1536` path is losing too much signal by rotating in `2048` space and then discarding
  the tail

That means:

- `H1` remains secondary
- `H2` is still partly true at the product level, because the current quantized objective does not
  match fp32 truth closely enough
- but the clearest concrete cause under `H3` is now storage/transform formulation, not graph
  traversal and not score-path divergence

## Suggested Next Slice

1. prototype tiled FWHT in the quantizer path for the `1536` compatibility tier
2. rerun the exact-only `1k` harness on that prototype before touching the graph again
3. if the prototype preserves most of the `~81%` reference result, carry it into the SQL/index path
   and rerun A4

## Review Focus

- whether the new padding/tail/tiled probes fairly identify transform-tail truncation as the main
  exact-only loss at `1536`
- whether tiled FWHT is now the right implementation direction for the next A4 slice
- whether any other cheaper quantizer-internal check should come before a tiled-FWHT prototype
