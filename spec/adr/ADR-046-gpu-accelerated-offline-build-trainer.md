---
id: ADR-046
title: "GPU-Accelerated Offline Build Trainer (Push-Model Artifacts)"
status: PROPOSED
impact: Affects ADR-030, ADR-032, ADR-033, ADR-036, ADR-037, ADR-038, ADR-045
date: 2026-04-19
---
# ADR-046: GPU-Accelerated Offline Build Trainer

## Context

Several proposed or accepted quantizer and index designs involve
substantive training work over the corpus:

- **OPQ** (ADR-036) — iterative rotation ↔ k-means codebook
  refinement.
- **Additive / Residual Quantization** (ADR-037) — joint
  optimization of M stacked codebooks.
- **LSQ** (ADR-038) — iterated local search at encoding plus
  joint codebook refinement.
- **SymphonyQG** (ADR-045) — quantization-aware graph
  construction; initial k-NN graph build dominates at scale.

All four are embarrassingly parallel in ways GPUs are good at
(dense linear algebra, k-means, k-NN graph construction via
CAGRA/cuVS) and are expensive in ways that matter at scale.
Published reference implementations (FAISS-GPU, cuVS, CAGRA,
Microsoft's DiskANN GPU build) demonstrate 10–100× speedups on
builds for multi-million-vector corpora.

Ecaz ships as a Postgres extension. Linking CUDA into the
server process is architecturally hostile: it forces the database
host to carry a GPU driver, enlarges the trust surface, and
couples release cycles of the extension to NVIDIA's stack.

A **push model** solves this cleanly: training runs offline (on
the user's workstation, a workstation GPU, or a rented cloud
GPU), producing a portable artifact file. Postgres loads the
artifact via SQL; the server itself remains CUDA-free.

This mirrors the FAISS / ScaNN operational pattern and matches
the user's stated preference in the discussion that preceded
this ADR.

## Decision

Adopt a **push-model offline build trainer** as the official
path for GPU acceleration of training-heavy quantizers and index
builds. Concretely:

1. **`ecaz-train` CLI**, a separate binary outside the
   extension. Takes vectors as input, produces a versioned
   artifact file. Pluggable backend: CPU (default, no CUDA) or
   GPU (cuVS / FAISS-GPU / custom kernels).
2. **Stable artifact format**, versioned from v1. Self-describing
   header including: format version, quantizer type, dim, bits,
   seed, training-sample hash, backend tag, creation time.
3. **SQL load path.** `CREATE INDEX ... WITH (codebook =
   '/path/to/artifact')` or a `ecaz.load_training_artifact()`
   SQL function. Index build consumes the artifact instead of
   training in-process.
4. **CPU path remains authoritative.** Every artifact reproducible
   from the CPU backend; GPU is an optimization, not a new
   algorithm. Tests gate on bit-equivalence between CPU and GPU
   outputs (within documented numerical tolerance).
5. **No runtime GPU dependency.** The server never calls GPU code.
   Query path is unchanged.

### Artifact contents by quantizer

- **OPQ (ADR-036):** rotation matrix + PQ codebooks.
- **AQ/RVQ (ADR-037):** M additive codebooks + training metadata.
- **LSQ (ADR-038):** refined codebooks (wire-compatible with PQ,
  per ADR-038's scope).
- **SymphonyQG (ADR-045):** optionally a pre-built k-NN graph
  file (CAGRA output) consumed as a seed for CPU-side
  out-degree padding and RaBitQ-aware refinement.

### Non-goals

- **Sidecar / RPC trainer.** A localhost or networked service
  that trains on demand during `CREATE INDEX`. Considered and
  rejected as the default: introduces service discovery, auth,
  streaming, and failure-mode surface without materially
  improving on the push model for artifacts that are small and
  reused across builds. Can be revisited if automatic-build
  ergonomics become important.
- **Per-insert GPU encoding.** Live inserts remain CPU-only.
- **GPU-backed query path.** Graph traversal is
  pointer-chasing, low arithmetic intensity, latency-critical —
  CPU wins and the analysis is not close.

## Consequences

### What the extension contains

- Artifact reader/writer (pure Rust, no CUDA).
- SQL-level load path.
- Build routines that accept a preloaded artifact in place of
  training in-process.

### What `ecaz-train` contains

- Ingestion (reads vectors from file, from Postgres via libpq,
  or from stdin).
- CPU trainer (same algorithms that would otherwise run
  in-extension).
- Optional GPU trainer behind a feature flag (`--backend=gpu`),
  linking cuVS / FAISS / custom kernels. Packaged separately —
  distributors can ship `ecaz-train` with or without CUDA.
- Deterministic output given the same seed and sample (CPU).
  GPU output required to match CPU output within a published
  tolerance.

### Build-time cost

For training-heavy quantizers on large corpora, expected GPU
speedups:

- OPQ k-means + rotation: **10–50×** on consumer GPUs (3060 /
  3090 class).
- AQ / RVQ joint optimization: **20–80×** (more iterations, more
  to parallelize).
- LSQ refinement: **10–30×**.
- SymphonyQG initial k-NN graph (CAGRA): **20–100×** vs
  CPU hnswlib-style build.

Below ~1M vectors the CPU path is typically not the bottleneck
(heap scan + WAL dominate) and GPU offers little. The push model
makes this a user choice rather than an architecture commitment.

### Storage and compatibility

- Artifact files: KB (OPQ rotation + codebooks) to low GB (full
  CAGRA graph seed for 100M+ corpora). Transportable; users
  train on one machine and deploy to another.
- Artifact format versioned; older versions remain loadable by
  newer extension builds (per the same discipline as
  `INDEX_FORMAT_V*`).
- Mixed deployments (some indexes built from artifact, some
  built in-process on CPU) supported — they produce the same
  on-disk index format.

### Operational

- Security: artifacts are untrusted input. Loader validates
  header, enforces size and shape bounds, refuses mismatched
  dim/bits/quantizer-type. No code execution surface.
- Reproducibility: CPU backend is canonical. GPU runs include a
  `--verify-against-cpu` mode for release validation.
- Packaging: extension ships as today. `ecaz-train` ships as
  an adjacent crate with optional `gpu` feature; distributors
  can produce a CUDA-enabled build without touching the
  extension.

### User-visible change

```sql
-- Train offline:
--   ecaz-train \
--     --quantizer=opq --dim=1536 --bits=4 \
--     --backend=gpu \
--     --input=corpus.f32 \
--     --output=opq.artifact

CREATE INDEX ON t USING symphony (embedding)
  WITH (training_artifact = '/var/lib/ecaz/opq.artifact');
```

No change for users who never opt in — in-process CPU training
remains the default.

## Alternatives considered

### Link CUDA into the extension

Rejected. Couples the extension's release cycle to NVIDIA's,
enlarges trust surface, forces GPU on DB hosts, and provides no
user-visible win over the push model for the training workloads
involved.

### Sidecar trainer service (pull model)

Rejected as default. Adds service discovery, auth, streaming,
and failure-mode surface without a clear ergonomic win for
artifacts that are produced rarely and reused across many
builds. Remains available as a follow-on if
automatic-GPU-on-CREATE-INDEX becomes a priority.

### GPU encoding at insert time

Rejected. Per-tuple CUDA launch overhead (~5–10 μs) exceeds the
per-vector CPU encode cost (~1–3 μs) at ecaz's tuple sizes.
Would regress insert latency, not improve it.

### GPU on the query path

Rejected. HNSW/SymphonyQG traversal is pointer-chasing, low
arithmetic intensity, latency-critical. PCIe round-trip alone
exceeds current total query latency. The analysis is not close.

## References

- ADR-030: FastScan grouped subvector scoring
- ADR-032: Coexisting index formats
- ADR-033: Shared graph lifecycle format adapters
- ADR-036: OPQ rotation successor to SRHT
- ADR-037: Additive / residual quantization
- ADR-038: Local search quantization
- ADR-045: SymphonyQG quantized-graph access method
- NVIDIA cuVS / CAGRA — GPU-accelerated k-NN graph construction
- FAISS-GPU — reference implementations of OPQ, IVF-PQ, AQ on GPU
- Microsoft DiskANN — GPU-accelerated Vamana build path
