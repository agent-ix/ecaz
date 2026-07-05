# Task 20: OPQ Rotation Front-End for PqFastScan

Status: proposed — highest-leverage recall lever after task 16.

Executes ADR-036.

## Scope

Add Optimized Product Quantization (OPQ) as a transform option alongside
SRHT on the PqFastScan build path. OPQ jointly learns a rotation `R` and
grouped PQ codebooks such that quantization error under `R` is minimized.

Goal: +10–20% recall per byte at the same index size and the same scan
kernel. Opt-in via reloption initially; revisit default after measurement.

Wire format changes: the v2 metadata transform descriptor gains an
`OpqTransform` variant; query-time cost is unchanged (still one matrix
apply); build-time cost is materially higher (iterative training).

## Why this before AQ/RVQ

- **Kernel-compatible.** OPQ's output is still grouped PQ codes — the
  FastScan LUT scorer, binary sidecar, and hot/cold payout all work
  unchanged. AQ/RVQ is structural and opens the scoring-kernel question.
- **Drop-in upgrade.** Swap rotation train-side; runtime is identical.
- **Compounds through the scale ladder.** OPQ codes feed DiskANN and
  SPANN the same way they feed HNSW+PqFastScan. One kernel upgrade,
  three index families benefit.

## Design outline

See ADR-036 for details. Summary:

- **Training loop.** Alternate between (a) fixing R and training PQ
  codebooks via k-means, and (b) fixing codebooks and solving for R via
  SVD of the codebook-residual covariance. Converge in 20–50 iterations
  for 1536-dim / 96 subvectors × 16 codebooks.
- **Initial R.** PCA, then break ties with balanced permutation
  assignment across subvectors. Matches the "Non-Parametric OPQ"
  variant in the original Ge et al. paper; typically better end recall
  than random-init OPQ at the 96-subvector granularity.
- **Runtime.** Apply R once at query prep; identical from there on.

## Subtasks

### Metadata and training

- [ ] **Transform descriptor extension.** Add `OpqTransform { rotation:
  Box<[f32]> }` to the v2 transform enum in `src/am/metadata.rs` (or
  wherever task 14 landed the versioned descriptors). Serialization in
  the metadata page treats OPQ as a dense NxN float matrix in fixed
  byte order.
- [ ] **Training harness.** `src/quant/opq_train.rs`. Takes a training
  corpus sample, initial rotation hint, and the same
  `grouped_pq_train(...)` call the SRHT path uses today. Emits R plus
  the final codebooks.
- [ ] **Determinism.** Seed-equivalent rebuilds must produce identical
  R and codebooks. Same rule as grouped k-means determinism (task 15
  carried this forward).

### Build path

- [ ] **Reloption plumbing.** Add `transform='opq' | 'srht'` to
  `src/am/options.rs`. Default remains `srht`. Validate at CREATE INDEX.
- [ ] **Flush dispatch.** `flush_build_state` routes to OPQ training
  when `transform='opq'`. Output codes go through the same PqFastScan
  flush path as SRHT.
- [ ] **Build-time budget.** OPQ training is 20–50x slower than SRHT at
  the training-sample size. Cap total train time with a loop budget
  (default 30 iterations); emit a log line at completion.

### Scan path

- [ ] **Query prep.** `prepare_ip_query` dispatches on the transform
  descriptor. OPQ just replaces the SRHT apply with a dense GEMV; no
  FWHT. Prep cost shifts from `O(n log n)` to `O(n²)` but `n²` at
  1536-dim is ~9M FLOPs (trivial).
- [ ] **Zero scan-kernel change.** The hot LUT scorer must stay
  identical. Confirmed by reusing the existing
  `score_grouped_fastscan_*` path unchanged.
- [ ] **Cold rerank compatibility.** Rerank payload is
  rotation-invariant for the inner product op (OPQ rotation is
  orthonormal). Confirm no rerank path needs a reverse rotation.

### Measurement

- [ ] **Instrumentation.** Baseline packet: recall and latency at same
  byte budget on 50k warm real seam with `transform='srht'` vs
  `transform='opq'`.
- [ ] **Per-dim target.** Validate on both 1536 and 768 dim seams.
  OPQ's win typically grows with dim (more subvectors = more room for
  cross-subvector alignment).
- [ ] **Decision gate.** If OPQ doesn't beat SRHT by ≥5 pp recall at
  the same byte budget on either seam, reject as default-worthy and
  keep opt-in. This is a measurement checkpoint, not a hard cancel.

### Docs

- [ ] **README.** When to pick OPQ: anyone willing to trade build time
  for recall. When to keep SRHT: rapid-rebuild or training-corpus-
  scarce workloads.
- [ ] **ADR-036 status update.** PROPOSED → DECIDED on measurement
  green.

## Owns

- ADR-036
- `src/quant/opq_train.rs` (new)
- OPQ branch of `prepare_ip_query`

## Dependencies

- Task 15 (PqFastScan first-class). OPQ is a build-side swap on the
  PqFastScan format; makes no sense before that format is stable.
- Task 14 versioned transform descriptor. Landed as of the
  `adr030-v2-*` branch lineage.

## Unblocks

- A better recall/byte curve on PqFastScan without wire-format change.
- A natural "quality mode" reloption for users who care about recall
  more than build time.
- Best foundation for AQ/RVQ (task 22) to build on — OPQ first validates
  the rotation+codebook pipeline end to end.

## Out of scope

- AQ/RVQ (separate task; structural format change).
- Anisotropic vector quantization (ScaNN-style; separate ADR if ever).
- OPQ on TurboQuant. TurboQuant already has SRHT+Lloyd-Max; OPQ there
  would need a different integration story and isn't prioritized.

## Notes

- **Build time is the real cost.** For 1M training vectors, OPQ takes
  tens of minutes vs sub-minute for SRHT. Practical for one-shot
  builds, uncomfortable for rapid iteration during development.
- **Runtime is free.** Query-side is just a matrix apply, same as SRHT.
- **NPQ baseline.** The "Non-Parametric OPQ" init (PCA + balanced
  permutation) usually gets 80% of the win with 20% of the iterations.
  Consider that as a fast default if full OPQ training is painful.
- **Not a silver bullet for short vectors.** At very low dim (128),
  OPQ over SRHT is a small win. The recall delta grows with dim.
