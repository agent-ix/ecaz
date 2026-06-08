# Task 87 Phase 1 Design Revision

This packet resolves the four blockers in
`reviews/task-87/001-phase1-design/feedback/2026-06-08-01-reviewer.md`.

## B1: HNSW TurboQuant Exact-Score Modes

Phase 5 HNSW is scoped to batch **only** the `FullLut` no-QJL 4-bit mode:

- `TurboQuantExactScoreMode::FullLut` routes through
  `score_ip_from_parts_lut_no_qjl_4bit` and is the mode Task 87 will
  batch.
- `TurboQuantExactScoreMode::TiledLut` remains inline until a follow-up
  proves that tiled batches preserve the intended tile-local reuse.
- `TurboQuantExactScoreMode::Int8Approx` remains inline until a follow-up
  decides whether approximate int8 scoring belongs in the exact
  no-QJL 4-bit acceptance gate.

Phase 5 validation language is therefore:

- HNSW `FullLut` TurboQuant no-QJL 4-bit recall must be byte-equal at
  every measured cell versus pre-batch `FullLut`.
- HNSW `TiledLut`, `Int8Approx`, and generic exact fallback must show no
  regression on focused smoke coverage, but they are not counted as the
  Task 87 batched scoring-share win.

This keeps the Phase 5 slice narrow and avoids silently broadening Task
87 into multiple HNSW exact-score-mode projects.

## B2: DiskANN Scope Decision

Task 87 will **not** reinterpret DiskANN grouped-PQ or RaBitQ as satisfying
the TurboQuant no-QJL 4-bit gate.

Decision:

- Phase 4 is treated as a pre-declared DiskANN Stop Condition for the
  current branch: there is no DiskANN TurboQuant search codec to route
  through `CandidateBatch`.
- The follow-up task is now opened as
  `plan/tasks/90-diskann-turboquant-search-codec.md`.
- Task 90 owns the source audit and any narrow, on-disk-format-neutral
  DiskANN TurboQuant search-code enablement.
- If Task 90 lands a codec before Task 87 closes, a later Task 87
  DiskANN packet may consume it. Until then, Task 87 Phase 4 remains a
  documented Stop Condition rather than a grouped-PQ/RaBitQ substitution.

Evidence:

- Task 86 packet 006 explicitly limited DiskANN coverage to the inspected
  grouped-PQ/RaBitQ adapter and noted that a separate DiskANN TQ path
  should be mapped before cross-index TQ design claims.
- Task 87's quantizer scope says kernel work is TurboQuant no-QJL 4-bit
  only.
- `reviews/task-87/001-phase1-design/artifacts/source-scoring-map.md`
  anchors the current DiskANN source surface:
  `DiskannBuildCodec::{PqFastScan, RaBitQ}` and
  `DiskannPreparedPrefilter::{BinarySidecar, GroupedPq, RaBitQ}`.

## B3: HNSW Borrow Source

HNSW will not require `CandidateBatch<'a>` to hold multiple page/tuple
borrows from `load_exact_graph_element` concurrently.

Phase 5 HNSW will use an HNSW-local owned scratch path:

1. For one successor expansion, load each eligible neighbor graph
   element as today.
2. Copy only the score code bytes and metadata needed for the in-scope
   `FullLut` scorer into a fixed-size `Vec` scratch owned by the HNSW
   expansion frame.
3. Push `CandidateBatch` entries that borrow from that owned scratch,
   not from transient page/tuple backings.
4. Flush the batch before the expansion frame returns.

This intentionally accepts a small copy cost in HNSW. The HNSW batch
sizes are bounded by layer neighbor slots (`m` or `2m`), so correctness
and lifetime clarity matter more than avoiding every byte copy. The copy
cost is part of the Phase 5 measured scoring-share outcome.

SPIRE and IVF may continue to borrow directly from contiguous column or
posting-list storage. The shared `CandidateBatch` contract remains a
borrowed view; HNSW simply provides an owned backing for that view.

## B4: SPIRE Structural Slice Gate

Phase 2 SPIRE is explicitly a **structural batching slice**.

The current SPIRE path already has a chunked batch loop in
`SpirePreparedAssignmentScorer::score_batch_ip`, and the initial shared
`CandidateBatch` flush may still call the same per-candidate LUT scorer.
Therefore Phase 2's acceptance gate is:

- recall byte-equal at every measured cell;
- no end-to-end latency regression beyond measurement noise;
- storage unchanged;
- non-batched SPIRE quant paths unchanged;
- the SPIRE scoring site uses the shared `CandidateBatch` abstraction
  for TurboQuant no-QJL 4-bit.

The `>= 2x` scoring-share target is moved to the first packet that lands
a real batch kernel, such as a 32-vector u8 nibble LUT scorer. Phase 2
must still record scoring-share timing, but a small or zero speedup does
not trigger the no-op Stop Condition as long as the structural route is
byte-equal and non-regressing.

## RaBitQ Metadata Note

`CandidateMeta::RaBitQ` is empty in the Phase 1 sketch because the
current IVF/DiskANN RaBitQ prepared scorer carries the rotation and
estimator metadata in `PreparedEstimator`.

If a future RaBitQ block kernel needs per-candidate side metadata, it
should add a typed metadata payload to the enum without changing the
`CandidateBatch` id/code/score-buffer contract.

## Revised Phase Sequence

1. Phase 2 SPIRE: structural `CandidateBatch` route for TQ no-QJL 4-bit.
2. Phase 3 IVF: `CandidateBatch` route for TQ no-QJL 4-bit posting-list
   chunks.
3. Phase 4 DiskANN: Stop Condition packet unless Task 90 lands a narrow
   TurboQuant DiskANN search codec first.
4. Phase 5 HNSW: `FullLut`-only structural `CandidateBatch` route with
   HNSW-owned code scratch.
5. Kernel packet: first real batched kernel and `>= 2x` scoring-share
   evidence on the AM surfaces where batch sizes justify it.

Task 87 remains incomplete until reviewer-approved packets cover every
accepted slice plus any accepted Stop Condition.
