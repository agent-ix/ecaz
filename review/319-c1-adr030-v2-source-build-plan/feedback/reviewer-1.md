## Feedback: ADR-030 v2 Source Build Plan

Read `plan_v2_grouped_source_build` in `src/am/build.rs`.

### What's right

- The plan type centralizes the v2-specific decisions: group size, subvector layout,
  training corpus sampling strategy, codec choices. That means downstream flush code
  can depend on a single struct rather than reaching into build-time constants.
- Planning is explicitly a source-column-required step. That matches the ADR-030 v2
  invariant: v2 builds cannot happen over an index-only input because training needs
  the source vectors.

### Concerns

1. **Plan → metadata mapping.** The plan must produce metadata fields that exactly
   match what the scan-side read code expects. The current path (packet 320) sets
   metadata from the plan, but there is no single test that verifies plan →
   metadata → scan descriptor round-trip. Worth adding.

2. **Plan immutability.** Once a plan is made, the flush should not be able to
   override `group_size` or `subvector_count`. If `BuildFlushOutput` currently takes
   those as separate parameters, there's a risk of drift. Derive them from the plan
   directly if not already.

3. **Training sample size.** The plan uses `MAX_TRAIN_SIZE = 1024`. At very large
   corpora, that's a tiny sample. At very small corpora, it may exceed the row count
   and training falls back to whatever happens when k-means sees fewer rows than
   `MAX_TRAIN_SIZE`. Check edge case explicitly, with a test for the small-corpus
   path — a build on a dev dataset of 100 rows is the first thing someone will try
   experimentally.

### Observation

Moving the per-build decisions into a plan struct is the right direction for getting
the constants eventually into user-facing options (`WITH (group_size=32)`) without
reshuffling everything.
