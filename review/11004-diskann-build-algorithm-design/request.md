# Review Request: DiskANN Build Algorithm Design

Branch: `adr034-diskann-access-method`

Scope:
- `plan/design/diskann-build-algorithm.md`

New directory: `plan/design/` (created by this packet).

## What this slice is

Design doc for the `tqdiskann` build pipeline (task 17 phase 2).
Describes the Vamana construction algorithm, cites
pgvectorscale/VectorChord/Microsoft-DiskANN source files, and
pins the distance-wrapper, medoid-approximation, and page-
persistence rules that phase 2 implementation will follow.

No code ships under this slice. Design doc only.

## What changed

New file: `plan/design/diskann-build-algorithm.md`.

Pipeline shape:

```
heap scan → SRHT → grouped-PQ4 training → grouped-PQ4 encoding
         → binary sidecar → Vamana build → medoid approximation
         → persist hot/cold pages → metadata page
```

Everything above the "Vamana build" line reuses existing tqvector
code. The new pieces are:

- Vamana two-pass α-pruning build (α=1.0 first, α=configured
  second).
- Random-sample medoid approximation at cap `S = 10 000`.
- Hot/cold page persistence in visit-order-from-medoid for
  scan-side cache locality.
- New metadata-page struct with `entry_point_tid`, `graph_degree_R`,
  `build_list_size_L`, `alpha` (`f32`, pgvectorscale-compatible),
  `format_version = INDEX_FORMAT_V3_DISKANN`.

Distance-function choice: negative inner product wrapped as
`d = max(0, -ip + C)` to keep α-dominance well-defined.

## Review focus

- **Distance wrapper.** The `d = max(0, -ip + C)` wrapper with
  `C = 1` assumes unit-normalized vectors. Reviewer should
  either confirm this matches the FastScan scoring contract in
  `src/quant/grouped_pq.rs` / `src/quant/prod.rs` or propose a
  different wrapper. Open question 1 in the design doc raises
  cosine-after-normalization as an alternative.
- **Reference source citations.** The doc cites
  `pgvectorscale/src/access_method/{build,graph,pruner}.rs`
  and `vectorchord/src/indexing/diskann.rs`. Reviewer should
  confirm these files exist at current HEAD of those
  repositories (citations were gathered from public project
  structure, not verified against specific commits).
- **Persistence order (visit-order-from-medoid).** This
  optimizes scan cache locality at the cost of a full BFS
  between build and persist. Reviewer should weigh whether the
  scan-side win justifies the build-time cost at 1B-scale.
  Design doc open question 3 flags this.
- **Determinism.** f32 rounding in `score_ip_codes_lite` may
  make α-dominance nondeterministic across runs even with a
  fixed seed. Design doc open question 4 asks reviewer to
  confirm this is acceptable (standard Vamana posture is that
  graphs are seed-deterministic but not exactly reproducible
  across machines).

## Questions to answer

- **Medoid sample cap.** pgvectorscale caps at 10 000 samples.
  Is this right for our 1536-dim/4-bit corpus, or should we
  probe higher because FastScan scoring is cheap? Design doc
  open question 2 flags this.
- **Recall gate at phase 3 scan.** The design proposes
  `Recall@10 ≥ 0.90` at `ef_search = 128`, `R = 32`, `α = 1.2`
  on the real 10k fixture as the build-quality bar. Reviewer
  should confirm this matches task 17 phase 3 exit criteria
  and NFR-003 expectations. A weaker bar (e.g., ≥ 0.85) may
  be appropriate for a first-pass v0 AM; a stronger bar
  (e.g., ≥ 0.95) may be needed to justify the AM at all given
  that `tqhnsw` already lands ≥ 0.92 on the same fixture.
- **Connectivity gate.** Design proposes a 95% BFS-from-
  medoid reachability threshold in the integration tests.
  pgvectorscale does not enforce this explicitly; is it a
  useful guard, or will it produce false alarms on sparse
  dense-data regions?

## Dependencies

- Task 15 (PqFastScan first-class) — landed on `main`, build
  pipeline consumes existing codebook/encoder code.
- ADR-034 (DiskANN as second AM, PROPOSED).
- ADR-042, ADR-043 (lock-ordering ADRs, PROPOSED in packets
  11002/11003).

## Companion packets

- `review/11001-diskann-task17-plan/` — task 17 plan (phase 2
  is the execution vehicle for this design).
- `review/11002-adr042-vamana-insert-lock-ordering/` —
  ADR-042 (live insert shares the `RobustPrune` helper that
  phase 2 build introduces).
- `review/11003-adr043-vamana-vacuum-lock-ordering/` —
  ADR-043 (step 10 defers medoid migration to rebuild, which
  reuses the medoid-approximation helper described here).

## Definition of ready (for design → frozen)

- Distance-wrapper choice (open question 1) resolved.
- Medoid-sample cap (open question 2) resolved or deferred.
- Persistence-order (open question 3) resolved.
- Determinism expectation (open question 4) confirmed or
  revised.
- Recall gate at phase 3 scan agreed with reviewer so phase 2
  can be validated against a stable bar.
