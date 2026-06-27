# Task 122 Packet 010: Closeout Keep Experimental

This is the Task 122 closeout request. The recommended outcome is:

```text
keep experimental / promote follow-up
```

TurboQuant should not be deferred outright. The evidence shows a viable
stage-2 pipeline shape, but it has only been measured in the CLI sidecar
harness, not landed as an in-engine AM path.

## Requirement Mapping

Phase 1, scorer parity:

- Packet 001 inventoried IVF, SPIRE, HNSW, and DiskANN TQ scorer paths.
- Production IVF/SPIRE no-QJL/QJL lanes have comparable batch/block scorer
  status for the measured lanes.
- Exact-dequant and some graph fallback paths remain scalar and were not used
  as first comparators.

Phase 2, fuse score/top-k/materialization:

- The SPIRE pre-materialization prune explored during this branch was split out
  of the Task 122 landing set after user clarification that Task 122 should stay
  TurboQuant-focused.
- No Phase 2 engine optimization is promoted by this TQ-only closeout.
- The TQ-specific follow-up is Task 124, which must implement score/top-k/final
  rerank fusion around the in-engine IVF TQ stage-2 path.

Phase 3, TQ as candidate reducer before f32 rerank:

- Packet 008 added the measurement harness.
- Packet 009 ran the 10k/50k/100k matrix.
- TQ stage-2 with `candidate_k=100` and exact f32 width `25` matched full-f32
  sidecar recall and reduced exact f32 work from 100 candidates to 25.

Phase 4, block-level pruning:

- No durable block-level upper-bound pruning rule was promoted in this task.
- This remains follow-up work if the in-engine stage-2 path needs more
  candidate-surface reduction.

Phase 5, quality only when it reduces rerank width:

- Packet 009 shows TQ quality is useful only at exact f32 width `25`.
- Width `10` is too narrow at 50k/100k, so it should not be promoted.

Phase 6, storage/IO cases:

- Packet 009 records base index storage and sidecar storage/bytes.
- At 100k, persisted sidecar sizes were:
  - f32: `585.94 MiB`
  - RaBitQ8: `147.63 MiB`
  - TurboQuant4: `73.62 MiB`
- TQ sidecar bytes touched were `75.39 KiB` for `candidate_k=100`, half RaBitQ8
  and one eighth of f32 sidecar bytes.

Phase 7, correct comparator matrix:

- Packet 009 covered RaBitQ frontier -> f32, RaBitQ frontier -> RaBitQ8 -> f32,
  and RaBitQ frontier -> TQ -> f32 at 10k/50k/100k.
- Direct SPIRE evidence is intentionally not part of the Task 122 landing set
  after the TQ-only split.

## Closeout Decision

Do not close Task 122 as a TQ failure. The best measured path is:

```text
RaBitQ frontier -> TurboQuant stage-2 reducer -> exact heap f32 width 25
```

At `candidate_k=100`, `final_rerank_k=25`, and `tid-sorted` sidecar reads:

```text
scale  nprobe  f32 recall/p95      TQ->f32@25 recall/p95
10k    32      1.0000 / 2.389 ms   1.0000 / 1.855 ms
10k    64      1.0000 / 2.898 ms   1.0000 / 2.176 ms
50k    32      0.9960 / 4.822 ms   0.9960 / 3.885 ms
50k    64      1.0000 / 7.835 ms   1.0000 / 6.819 ms
100k   32      0.9730 / 8.713 ms   0.9730 / 7.953 ms
100k   64      1.0000 / 13.815 ms  1.0000 / 13.517 ms
```

This supports a follow-up implementation task, not a product promotion inside
Task 122. The follow-up should implement the stage-2 path inside the AM
pipeline and preserve packet 009 as the benchmark gate.

## Recommended Follow-Up

Create a task for an in-engine IVF pipeline:

```text
ec_ivf RaBitQ candidate frontier -> TQ persisted/index-side stage-2 score -> exact heap f32 width 25
```

Minimum gates for that follow-up:

- 10k/50k/100k recall + latency + storage via `ecaz bench suite`.
- f32 fetch count and materialized row counters.
- Comparator rows against current RaBitQ + f32 rerank width 100 and RaBitQ8
  stage-2.
- Hot local and one IO-sensitive/cold-cache variant before claiming a product
  latency win.
