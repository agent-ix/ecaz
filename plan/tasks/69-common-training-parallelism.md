# Task 69: Common Training Parallelism (k-means + PQ4 codec)

Status: proposed
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (paired with Task 68; Task 68 slice 1 consumes this)

## Why

`crate::am::common::training` is shared build-time code called by
both IVF (`src/am/ec_ivf/build.rs:298,521`,
`src/am/ec_ivf/quantizer.rs:827`) and SPIRE
(`src/am/ec_spire/build/training.rs:12,142,99`,
`src/am/ec_spire/build/recursive.rs:30,40`,
`src/am/ec_spire/update/materialization.rs:392`,
`src/am/ec_spire/update/routing.rs:163`). Parallelising it lifts
both subsystems in one place, with no AM-specific changes required.

The current implementation is single-threaded:

- `train_spherical_kmeans` (`src/am/common/training.rs:71`) — the
  per-iteration assignment + accumulator loop at lines 109–119 is
  a textbook embarrassingly-parallel reduction over `samples`.
  Centroid renormalisation at lines 121–131 is a per-centroid
  loop, also parallelisable but smaller.
- `assign_vector_to_centroid` (`src/am/common/training.rs:144`) is
  a per-vector call that downstream code already invokes in a
  loop.
- `train_grouped_pq4_model` (`src/am/common/training.rs:278`)
  runs k-means independently per group; today every group is
  trained sequentially.

SPIRE compounds the win because recursion calls k-means at every
level (`ec_spire/build/recursive.rs:30`). IVF gets the same
constant-factor speedup on the single-level path. No
behaviour-visible API change is required as long as we keep
deterministic outputs at fixed seeds.

## Non-Goals

- Do not change the public training API surface in
  signature-incompatible ways. Existing call sites in IVF and SPIRE
  must compile unchanged unless the change is a strict opt-in
  parallel variant (e.g. `train_spherical_kmeans_parallel`).
- Do not introduce a thread pool outside of the existing rayon
  global pool. PG-side build threads are out of scope; this task
  is in-process CPU parallelism only.
- Do not change centroid initialisation, fallback selection, or
  the spherical-renormalisation step semantically.

## Scope

### Slice A — Parallel k-means iteration loop

Target: `train_spherical_kmeans`
(`src/am/common/training.rs:71-141`).

Required shape:

- Rewrite the iteration body (lines 104–135) so the assignment +
  partial-sum reduction (lines 108–119) runs in parallel over
  `samples` using rayon. Use a per-thread `Vec<Vec<f32>>` sums
  buffer plus a per-thread `Vec<usize>` counts buffer, reduced
  into the iteration-global `sums` and `counts` after the
  parallel section.
- Preserve determinism: at a fixed `seed`, fixed `nlists`, fixed
  `max_iterations`, and fixed input vector list, the returned
  centroids must be **bit-identical** to the scalar baseline.
  Achieve this by ordering the reduction deterministically (e.g.
  reduce per-thread partials in a fixed thread-id order, or use
  `.reduce_with` over indexed chunks).
- The `changed` flag must remain semantically equivalent. Compute
  it as an OR across the parallel pass.
- The empty-cluster fallback (lines 121–127) stays sequential —
  it touches a small `nlists`-sized loop and depends on iteration
  index.

Acceptance: a property test using a small seeded fixture asserts
byte-equal centroid output between the scalar reference (kept as a
private function, e.g. `train_spherical_kmeans_scalar`) and the new
parallel implementation across at least 16 seed × shape
combinations.

### Slice B — Parallel grouped PQ4 codebook training

Target: `train_grouped_pq4_model`
(`src/am/common/training.rs:278`).

Required shape:

- The per-group k-means calls inside the grouped trainer are
  independent. Rayon over groups, calling Slice A's k-means
  implementation per group.
- Determinism: identical model output to the scalar baseline at
  fixed seed.

Acceptance: a property test asserts byte-equal
`GroupedPq4Model.codebooks` and `.signs` between scalar and
parallel implementations across at least 8 seed × (dimensions,
group_size) combinations.

### Slice C — Parallel assignment fan-out helper

Add a new public-in-crate helper
`assign_vectors_to_centroids(error_label, sources: &[&[f32]],
model: &SphericalKMeansModel) -> Result<Vec<usize>, String>` that
parallelises the per-vector loop and short-circuits on the first
error in deterministic source order (the lowest source index
whose call returns `Err`).

Acceptance: existing IVF and SPIRE call sites that currently loop
over `assign_vector_to_centroid` per vector are migrated to the
new helper. The migration is a separate small commit per
subsystem so it is reviewable independently.

### Slice D — Measurement

A measurement packet under
`reviews/task-69/{NNN}-training-parallelism-measurement/`
recording, on M5 release-mode builds at the same two fixture
sizes used by Task 68 Phase 1:

- `train_spherical_kmeans` wall time at scalar vs parallel,
  recorded for at least two `(nlists, sample_count, dimensions,
  max_iterations)` shapes that match real IVF + SPIRE call sites.
- `train_grouped_pq4_model` wall time at scalar vs parallel.
- Byte-equality evidence for both (digest of the returned
  `SphericalKMeansModel.centroids` and `GroupedPq4Model.codebooks`).
- Worst-case core scaling: with `RAYON_NUM_THREADS=1`, the parallel
  implementation must not be more than ~10 % slower than the
  scalar baseline (i.e. no per-iteration overhead regression for
  single-thread consumers).

## Exit Criteria

- Slices A–C landed with reviewer-approved code packets.
- Slice D measurement packet shows a multi-× wall-time win at the
  M5 corpus shapes that match IVF + SPIRE call sites, **and**
  byte-equal model output vs scalar at fixed seeds, **and**
  ≤ 10 % regression at `RAYON_NUM_THREADS=1`.
- No new `unsafe { ... }` blocks introduced. The parallel
  reductions must be expressible in safe rayon — if a slice
  requires `unsafe`, write a design note first and BLOCK the
  slice for review.
- No behaviour change in IVF or SPIRE recall on the comparator
  fixtures (within the 0.5 pp recall floor each AM already
  documents).
- `cargo clippy --all-targets --no-default-features --features
  pg18 -- -D warnings` clean. Tests added under
  `crates/.../tests` or as `#[cfg(test)]` modules next to the
  training source.

## Coordination

- **Task 68 slice 1 consumes this task.** Task 68 Phase 1 may
  proceed in parallel, but Task 68 slice 1 only lands after this
  task closes.
- IVF call sites are migrated as part of Slice C but no IVF
  perf claims are made under this task — IVF wall-time evidence
  belongs in whatever IVF lane the team opens next.
- This task does not change `ProdQuantizer`, RaBitQ, or any
  AM-specific quantizer surface. Scope is strictly
  `src/am/common/training.rs` plus call-site migrations.
- Honor memory `feedback_dont_defer_safety_fixes` and
  `feedback_anti_pattern_b_unbounded_lifetime` in review.
