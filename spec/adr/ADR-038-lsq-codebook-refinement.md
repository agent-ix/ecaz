---
id: ADR-038
title: "Local Search Quantization as Codebook Refinement"
status: PROPOSED
impact: Affects ADR-030, ADR-032, ADR-037
date: 2026-04-18
---
# ADR-038: Local Search Quantization (LSQ)

## Context

PqFastScan trains its grouped PQ codebooks with **standard k-means**:
for each subvector group, cluster training vectors into 16 centroids,
minimizing within-cluster sum-of-squares. Encoding a new vector means
picking the nearest centroid per subvector independently.

**Local Search Quantization** (LSQ; Martinez, Clement, Hoos, Little,
ECCV 2016) improves this in two ways:

1. **Better codebook training.** LSQ jointly optimizes all codebooks
   rather than treating each subvector independently. For PQ this
   means iterative re-clustering that accounts for cross-subvector
   residuals.

2. **Better encoding.** For each vector, LSQ uses iterated local
   search over codeword assignments to find a combination that
   reconstructs the vector more accurately than the independent
   per-subvector nearest-centroid choice.

Published results: incremental recall improvements at the same bit
budget, typically 2–5% on top of k-means-trained PQ. Smaller than
OPQ's 10–20% gain, smaller than AQ's ~2× compression, but much
cheaper to adopt.

LSQ is a member of the AQ family conceptually (both frame encoding
as a joint optimization problem). In practice, within FAISS, it
appears as `IndexLocalSearchQuantizer` and is often used on top of
PQ structure rather than replacing it.

## Decision

tqvector will treat **LSQ as a low-priority codebook refinement
lever** for the PqFastScan pipeline. Unlike OPQ and AQ, LSQ does
not change the wire format, does not change the scoring kernel,
and does not require a new quantizer. It is a drop-in training
quality improvement.

### Scope

LSQ applies at two points:

1. **Codebook training.** Replace or augment the current k-means
   training pipeline with LSQ's iterated joint optimization. One-time
   cost at build, no runtime impact.

2. **Encoding (optional).** Use iterated local search when
   encoding vectors (both at build and at insert) to pick a better
   per-vector assignment. Runtime cost per encoding grows modestly;
   insert throughput regression must be validated.

### Priority

LSQ lands **after** tasks 15 and 16 and **after** OPQ (ADR-036).
Reasons:

- Small absolute gain (2–5% recall) relative to investment required.
- No format change — LSQ can be added without a REINDEX for new
  indexes; existing indexes keep their k-means codebooks unchanged.
- Most valuable *after* OPQ rotation is in place, because LSQ and
  OPQ are complementary (better rotation + better codebook
  assignment compound).

If AQ (ADR-037) is ever adopted, LSQ naturally subsumes into the
AQ encoding pipeline (AQ uses LSQ-style local search as its native
encoding method). LSQ-on-PQ is the cheap increment while AQ is the
expensive structural change.

### Not in scope for this ADR

- Changing from PQ structure to AQ structure. See ADR-037.
- Learned rotation. See ADR-036.
- Runtime scoring kernel changes. LSQ affects training and encoding
  only; `scan.rs` scoring code is untouched.

## Consequences

### Build-time cost

Modest increase. LSQ's iterated optimization per subvector group
runs on the training sample (typically 1K–10K vectors per subvector
group in our current sizing). Total additional training cost:
~1.5–3× k-means training alone, well within the build-time budget.

### Insert-time cost

Optional. If encoding uses iterated local search per tuple, each
insert re-encodes the vector with a small inner loop. Expected
overhead: 1.2–1.5× current PqFastScan insert cost. If insert
throughput NFR is tight, encoding can stay at fast nearest-centroid
while training still uses LSQ.

### Runtime query cost

Zero. Codes and scoring kernel unchanged.

### Wire format

Unchanged. LSQ produces the same 4-bit grouped PQ codes that
k-means training produces; the difference is in which centroids
are chosen. No wire version bump. Existing PqFastScan indexes
rebuilt with LSQ training are byte-identical in layout to k-means
ones.

### Recall impact

Expected 2–5% recall improvement at same bits on published
benchmarks. We would validate on our own 50k real seam and larger
corpora before shipping as default.

### Compatibility

LSQ-trained codebooks are readable by any PqFastScan scan code
(no scanner changes). Mixed deployments (some indexes k-means, some
LSQ) work correctly.

## Alternatives considered

### Stay on k-means

Acceptable. LSQ's gain is small enough that skipping it is
defensible, especially if engineering capacity is the binding
constraint.

### Adopt AQ instead (ADR-037)

Strictly larger win (~2× compression). But much larger
implementation investment. If we are going to invest in encoding
complexity, AQ has higher ceiling than LSQ.

### OPQ + k-means (ADR-036 without this ADR)

Captures most of the "better quantization" improvement (10–20%
from OPQ) without the encoding-time cost of LSQ. Reasonable
simplification if LSQ's marginal gain doesn't justify the
additional encoding work.

### OPQ + LSQ (both ADRs)

Expected recall improvement stacks: ~12–25% over current k-means +
SRHT. Still short of AQ's ~2× compression win but without the
structural changes AQ requires. A reasonable "mid-complexity"
endpoint if AQ is deferred or rejected.

## References

- ADR-030: FastScan Grouped Subvector Scoring
- ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan
- ADR-036: OPQ Rotation
- ADR-037: Additive / Residual Quantization
- Martinez, Clement, Hoos, Little, "Revisiting Additive
  Quantization" / "LSQ" (ECCV 2016)
- FAISS `IndexLocalSearchQuantizer`
