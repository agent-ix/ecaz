# Task 205 review request: attribution closeout

This packet records the final disposition of the remaining attribution issue
on the corrected bounded-L rerun. It is a documentation-only checkpoint; it
does not reimplement or rerun Task 205.

The existing nine-arm PG18 matrix is complete at 10k/50k/100k and already has
outside interim acceptance of the mechanism. Its evidence proves threshold
activation and aggregate merged-batch pruning, but the committed physical
rows do not split threshold drops from post-threshold L-limit truncation. The
remote decode path records zero in the per-response field, so an exact split
cannot be recovered honestly from the existing artifacts.

The durable disposition is therefore:

1. accept the measured result as a combined bounded-L Algorithm 1 mechanism
   result, with recall/storage/topology evidence cited from packet 004;
2. withdraw any threshold-only, limit-only, or causal end-to-end attribution;
3. carry the limitation into Tasks 215 and 216; and
4. require any exact split to be a separately scoped instrumentation/arm
   change, not a reimplementation hidden in this closeout.

Evidence and the reasoning are in `artifacts/attribution-disposition.md`; the
provenance is in `artifacts/manifest.md`.
