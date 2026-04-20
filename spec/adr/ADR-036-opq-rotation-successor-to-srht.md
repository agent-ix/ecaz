---
id: ADR-036
title: "OPQ Rotation as Successor to SRHT in the PqFastScan Pipeline"
status: PROPOSED
impact: Affects ADR-006, ADR-024, ADR-030, ADR-032, ADR-046
date: 2026-04-18
---
# ADR-036: OPQ Rotation for PqFastScan

## Context

The PqFastScan pipeline currently uses **SRHT** (Subsampled Randomized
Hadamard Transform) as its rotation front-end (ADR-024). SRHT is
**data-oblivious**: a deterministic random rotation seeded at build
time, independent of the corpus distribution. It decorrelates
coordinates well enough that per-subvector PQ codebooks trained on
the rotated data are reasonably effective.

**Optimized Product Quantization** (OPQ; Ge et al., CVPR 2013) takes a
different approach: it jointly learns the rotation matrix *and* the PQ
codebooks, iterating between rotation and codebook updates until
subvector quantization error is minimized. The rotation is no longer
random; it specifically decorrelates subspaces in a way the PQ
codebooks can exploit.

Measured impact in FAISS and published benchmarks: **10–20% better
recall at equivalent bit budget** compared to random rotation + PQ.
FAISS has shipped OPQ since 2014 (`IndexPreTransform` +
`IndexPQ`).

ADR-030's out-of-scope section explicitly calls OPQ a natural
follow-on. This ADR formalizes that as a proposal rather than an
aside.

## Decision

tqvector will treat **OPQ as the successor to SRHT** in the PqFastScan
pipeline, landing after tasks 15 and 16 and after DiskANN (ADR-034) is
stable. OPQ becomes the rotation front-end; the rest of the pipeline
(grouped PQ4, FastScan SIMD scoring, binary prefilter, hot/cold split)
is unchanged.

### When to land OPQ

OPQ is deliberately not part of the task-15 PqFastScan landing. Reasons:

- The 10–20% recall-per-byte win matters most at scales where bytes
  bind (1B+). Our current user cohort is below 500M.
- OPQ adds training-pipeline complexity: alternating optimization,
  potentially multiple training iterations, sensitivity to
  initialization.
- Task 15 is scoped around "PqFastScan as a peer format" — changing
  the rotation simultaneously would conflate two distinct changes.

OPQ becomes load-bearing when:

- ADR-034 (DiskANN) lands and pushes tqvector into the 500M–3B band,
  where index-size bytes are more expensive and recall-per-byte
  improvements translate to fewer rerank reads.
- ADR-035 (SPANN) is under serious consideration, because SPANN's
  replication multiplier rewards every byte of recall improvement.

### Relationship to TurboQuant

TurboQuant's SRHT + Lloyd-Max MSE codebook is not replaced by OPQ.
TurboQuant's codebook is derived from the theoretical Beta
distribution that rotation guarantees; its math is orthogonal to OPQ.
If TurboQuant survives as a default format past ADR-032, it keeps
SRHT. OPQ targets the PqFastScan path specifically.

### Relationship to AQ/RVQ

OPQ is a rotation front-end for PQ-family quantizers. AQ/RVQ
(ADR-037) is an alternative *compression* scheme that replaces PQ
with additive codes. The two are orthogonal: AQ can use OPQ as
its rotation front-end (often called "OPQ+AQ" in the literature),
or AQ can use a random rotation.

If both ADR-036 and ADR-037 proceed, the expected sequencing is
OPQ first (because it's a drop-in swap for SRHT and preserves the
PqFastScan scoring kernel), AQ second (because it changes the
scoring kernel itself).

## Consequences

### Build-time cost

OPQ training is iterative:

- Alternates between rotation update (closed-form, O(d² n)) and
  codebook update (k-means, O(c d n / k) per subvector).
- Typically 10–25 outer iterations to converge.
- Total build time increase relative to SRHT+PQ training: ~2–5×.

At task 15's build-time budget this is non-trivial but acceptable.
At billion-scale with subsampled training sets, it is bounded and
well within normal operational expectations.

### Runtime cost

**Zero.** OPQ produces a rotation matrix just like SRHT does. Query
rotation cost is identical (one matrix-vector multiply, which is
already SIMD-accelerated). No change to the scoring kernel.

### Rebuild trigger

OPQ is corpus-trained; distribution shift invalidates the rotation
more than SRHT does. The rebuild discipline already required for
PqFastScan's PQ codebooks covers this — both would be retrained
together.

### Storage implications

None. OPQ adds no per-vector bytes; the rotation matrix is a single
per-index artifact (a few MB at 1536d).

### GPU acceleration (optional)

OPQ training — alternating rotation update (closed-form SVD /
Procrustes over a d×d scatter) and per-subvector k-means — is
well-matched to GPU execution. Reference implementations in
FAISS-GPU and cuVS report **10–50×** speedups over CPU on
consumer GPUs (3060 / 3090 class) for multi-million-vector
training samples.

tqvector will expose this through the push-model offline
trainer defined in ADR-046: `tqvector-train --quantizer=opq
--backend=gpu` produces a portable artifact (rotation matrix +
codebooks) that the extension loads at `CREATE INDEX` time. The
server itself remains CUDA-free; the CPU trainer is
authoritative and GPU output is gated on bit-equivalence (within
documented numerical tolerance).

GPU acceleration is not required to adopt OPQ. At corpora below
~1M vectors CPU training is already acceptable; the GPU path
exists to keep OPQ viable at SPANN-era (ADR-035) scales where
training sample sizes grow into the tens of millions.

### Compatibility

New wire-format bump required: OPQ-rotated PqFastScan indexes are
not readable by SRHT-rotation code. Treat as `INDEX_FORMAT_V4_*` or
a `rotation = 'opq' | 'srht'` reloption on the existing PqFastScan
format. Migration path: REINDEX, same as any PQ retraining.

## Alternatives considered

### Keep SRHT

Defensible if tqvector's scale target stays below ~500M vectors,
where the 10–20% recall-per-byte difference doesn't meaningfully
affect latency or storage budget. Makes OPQ optional rather than
required.

### Adopt AQ/RVQ first

Larger compression win (~2× vs 10-20%), but changes the scoring
kernel. Riskier and longer to ship. OPQ is the safer increment.

### Learn rotation jointly with AQ (OPQ-AQ)

Superset of both ADR-036 and ADR-037. Ultimate end-state but
composite project; not appropriate as a single decision.

## References

- ADR-006: Own quantizer implementation based on TurboQuantDB
- ADR-024: FWHT transform strategy (SRHT rotation)
- ADR-030: FastScan Grouped Subvector Scoring
- ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan
- ADR-037: Additive / Residual Quantization (proposed peer)
- ADR-038: Local Search Quantization codebook refinement (proposed peer)
- ADR-046: GPU-accelerated offline build trainer
- Ge et al., "Optimized Product Quantization" (CVPR 2013)
- FAISS documentation: `IndexPreTransform + OPQMatrix + IndexPQ`
