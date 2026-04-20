---
id: ADR-037
title: "Additive / Residual Quantization as PqFastScan v2 Compression"
status: PROPOSED
impact: Affects ADR-030, ADR-032, ADR-034, ADR-035, ADR-036, ADR-046
date: 2026-04-18
---
# ADR-037: Additive / Residual Quantization (AQ / RVQ)

## Context

PqFastScan's current compression is **Product Quantization**: split
the rotated vector into subvectors, quantize each independently with
its own 16-centroid codebook, concatenate 4-bit codes. At 1536d with
group_size=16, that's 48 bytes per vector.

**Additive Quantization** (AQ; Babenko & Lempitsky, CVPR 2014) and
its variant **Residual Vector Quantization** (RVQ) use a different
structure: a vector is represented as the **sum** of M codebook
entries, one from each of M codebooks:

```
v ≈ c_1[i_1] + c_2[i_2] + ... + c_M[i_M]
```

Each codebook has K entries (typically K=256 → 8-bit codes).
Encoding requires M ⌈log₂ K⌉ bits per vector. Training jointly
optimizes all M codebooks to minimize reconstruction error.

Published results (FAISS, various papers) report **~2× more compact
than PQ at equivalent recall** — a 48 B PQ code can be matched by a
~24 B AQ code at similar recall quality. This is a much bigger win
than OPQ's 10–20%.

**Local Search Quantization** (LSQ; Martinez et al., ECCV 2016) is a
refinement of AQ: uses iterated local search during encoding to
find better codeword assignments. Improves recall further at the
same bit budget. Covered under ADR-038 as a separate lever.

FAISS implements AQ variants (`IndexResidualQuantizer`,
`IndexLocalSearchQuantizer`) and their FastScan-compatible analogs.

## Decision

ecaz will **evaluate AQ/RVQ as PqFastScan v2** — a potential
successor compression scheme for the scoring kernel. This is the
most speculative of the proposed frontier ADRs: the win is large,
but it is not a drop-in replacement.

### Why "evaluate" rather than "adopt"

Three concerns that require empirical validation before commitment:

1. **FastScan compatibility.** PQ's FastScan exploits the fact that
   subvector codebooks are independent — the LUT is `[subvector][code] →
   partial_score`, and a vector's total score is a sum of
   independent lookups. AQ's additive structure is *also* a sum of
   codebook lookups, so it is FastScan-compatible in principle, but
   the exact SIMD shape differs and requires verification on our
   target architectures (AVX2, AVX-512, NEON).

2. **Encoding cost.** AQ encoding requires solving an
   M-codebook-assignment problem per vector. Beam search or iterated
   local search at encoding time is typical. Build time is ~5–10×
   slower than PQ's straightforward quantization per subvector.
   Acceptable at offline build; problematic for insert throughput.

3. **Scoring recall cliff.** Empirically AQ wins on
   recall-per-byte, but the margin narrows at the extreme low-bit
   regime (4–5 bits/vector-component-equivalent) that we operate in.
   Validation requires real-corpus benchmarks at our bit budgets.

### When the evaluation matters

AQ's value proposition is "same recall, half the bytes." That
compounds most aggressively through:

- **SPANN replication** (ADR-035). 8× replication of a 24 B AQ code
  costs 192 B per vector total; the same replication on a 48 B PQ
  code costs 384 B. AQ could cut SPANN's storage cost in half
  relative to PqFastScan-as-designed.
- **DiskANN page cache** (ADR-034). Smaller codes → more graph fits
  hot. AQ roughly doubles the effective graph-in-cache ratio.

Both of these are long-horizon ADRs. AQ's value scales with them.

### Relationship to other ADRs

- **ADR-036 (OPQ)** is compatible with AQ. The common stack in FAISS
  is OPQ rotation → AQ codes. Both can land; OPQ first since it is
  lower-risk.
- **ADR-038 (LSQ)** is an AQ-family refinement. If we adopt AQ, LSQ
  is the natural encoding-quality upgrade.
- **Coexists with PqFastScan (PQ)**, does not replace it. PQ would
  remain the primary format for HNSW and DiskANN at moderate scale;
  AQ becomes attractive at SPANN's billion-plus regime.

## Consequences

### Structural work required

- New quantizer implementation. Materially different from grouped
  PQ training — not an incremental change to `grouped_pq.rs`.
- New scoring-kernel code path in `am/scan.rs`. Different LUT
  layout than PqFastScan even if both are SIMD-amenable.
- New wire format: `INDEX_FORMAT_Vn_AQ` or similar. Cannot coexist
  in-place with PqFastScan codes.
- Training-pipeline complexity: joint codebook optimization, beam
  search at encoding.
- Insert-cost implications: per-tuple encoding is 5–10× slower than
  PQ. Insert throughput NFR would need revalidation.

### Build-time cost

Substantial. AQ training on a billion-vector corpus:

- Sampled training (typical ~1M vectors).
- Alternating optimization: residual assignment → codebook update,
  ~25–50 iterations.
- Per-iteration cost larger than PQ's k-means.
- Estimated 5–10× PQ's training time on the same sample.

For the target SPANN regime, this is acceptable as offline build
cost. For live insert, per-tuple encoding needs careful engineering
(precomputed residual trees, bounded beam width).

### Runtime cost

Scoring LUT shape differs. Expected ~1.5–2× slower per candidate
than PqFastScan at equivalent SIMD width. Offset by ~2× fewer
candidates needed (better recall per byte), so net-neutral to
slightly faster at equal recall target.

### Storage implications

~2× smaller index at equivalent recall. Compounds through replication
factors — most impactful in SPANN.

### GPU acceleration (optional)

AQ / RVQ training — joint optimization of M stacked codebooks
via alternating residual assignment and codebook update — is the
highest-leverage GPU target of any proposed quantizer. FAISS-GPU
and cuVS report **20–80×** speedups over CPU on consumer GPUs;
the gap widens with M because per-iteration work scales with the
number of codebooks.

Encoding per vector is the harder part: beam search over M
codebooks is branchy and latency-sensitive. GPU helps only for
batched offline encoding (full-corpus reencode during
`CREATE INDEX` or `REINDEX`). Per-tuple insert encoding remains CPU.

ecaz exposes GPU training through ADR-046's push-model
trainer: `ecaz-train --quantizer=aq --backend=gpu` produces
an artifact containing the M codebooks and training metadata,
loaded at `CREATE INDEX` time. The extension remains CUDA-free;
the CPU trainer is canonical; GPU output gates on
CPU-equivalence within documented tolerance.

At the SPANN / billion-scale regime where AQ's value
proposition matters most, the GPU path is the difference
between builds that finish overnight and builds that finish in
a work-week. Below ~10M vectors the CPU path is acceptable.

### Migration path

REINDEX only; no auto-upgrade. Consistent with the
no-auto-migration posture already established in ADR-030 and
ADR-032.

## Alternatives considered

### Stay on PQ

The simplest path. Acceptable if ecaz's scale targets stay
below 1B and/or SPANN is not pursued. AQ's value proposition is
almost entirely realized in the scale bands where storage bytes
compound.

### Adopt OPQ only (ADR-036, without this ADR)

Gets ~10–20% recall improvement without changing the scoring
kernel. Lower-risk. Acceptable if ADR-035 (SPANN) does not
proceed.

### OPQ + AQ together

The endpoint if both ADRs land. Order matters: OPQ first
(preserves PqFastScan scoring kernel), then AQ (changes it).

### Learned scalar quantization (alternative direction)

Several recent papers (e.g., ScaNN's anisotropic quantization)
explore learned scalar methods that are not in the PQ family.
Potentially competitive but with different implementation
trajectories and no FAISS reference implementation. Out of scope
for this ADR; could be a future comparison once AQ evaluation
completes.

## References

- ADR-030: FastScan Grouped Subvector Scoring
- ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan
- ADR-034: DiskANN as Second Access Method
- ADR-035: SPANN as Third Access Method
- ADR-036: OPQ Rotation
- ADR-038: Local Search Quantization
- Babenko & Lempitsky, "Additive Quantization for Extreme Vector
  Compression" (CVPR 2014)
- Chen et al., "Quantization for Approximate Nearest Neighbor
  Search" — RVQ treatments
- FAISS `IndexResidualQuantizer`, `IndexLocalSearchQuantizer`
- ADR-046: GPU-accelerated offline build trainer
