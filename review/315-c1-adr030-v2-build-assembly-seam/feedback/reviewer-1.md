## Feedback: ADR-030 v2 Build Assembly Seam

Read `V2GroupedBuildPayload`, `V2GroupedStagedChain`, `V2GroupedBuildPlan`, and
`BuildFlushOutput` in `src/am/build.rs`.

### Shape of the seam

The staging layer is the right abstraction: build up grouped payloads in memory, then
flush them through the existing DataPageChain rather than rolling a parallel builder.
That matches the approach taken in 314 for page placement.

### Concerns

1. **Duplicated encoder.** `encode_grouped_pq` lives in both `src/am/build.rs` and
   `src/bin/approx_score_study.rs` with independently maintained packing. The 313 tuple
   contract only works if both produce identical bytes. Consolidate to one encoder
   (probably in a module shared by build/bin/scorer) before the scorer packet, because
   a packing mismatch between build and score will look like recall noise, not a bug.

2. **Constant surface area.** `ADR030_EXPERIMENTAL_GROUP_SIZE = 16`, `MAX_TRAIN_SIZE =
   1024`, `KMEANS_ITERS = 8` are all compile-time constants inside `build.rs`. That's
   fine for the experimental gate but means per-index tuning requires a rebuild. Worth
   moving these into the metadata page (at least `group_size`) before the gate is
   lifted, so that a built index can self-describe its grouping.

3. **Train corpus sampling.** At 1024 rows, k-means training on wide vectors is tight.
   For a 100k-row index with 1536-dim, that's ~0.01% of the data. If the input
   distribution is multimodal, the training sample can miss modes. Worth recording what
   the sampling strategy is so downstream recall regressions can be traced back.

### What's well-scoped

The packet intentionally does not touch the flush output. That means build assembly can
be exercised in isolation (staging-layer tests) before the write path is wired. Good
incremental strategy.
