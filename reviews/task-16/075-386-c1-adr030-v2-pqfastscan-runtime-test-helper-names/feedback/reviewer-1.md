## Feedback: PqFastScan Runtime Test Helper Names

Read the renamed helpers in `src/lib.rs`.

### What's right

- **Narrow, mechanical rename of the shared runtime-fixture helper
  surface.** `create_grouped_v2_runtime_fixture* →
  create_pq_fastscan_runtime_fixture*`, plus the query/source/observed
  helpers. Keeps the helper layer aligned with the product name.
- **Deliberately scopes out the wider pg-test surface.** 388 picks
  that up, and splitting the two makes each diff actually reviewable.
  The shared helpers are the inner dependency; renaming them first
  means 388 only has to rewire test-function names, not also pivot
  the helper layer under them.
- **No behavior change.** `cargo check --tests` + clippy are
  sufficient for rename-only slices where the compiler enforces
  correctness.

### Concerns

1. **SQL table/index names still say `grouped_v2` inside the
   helpers.** That's flagged as intentional, but means the new
   `create_pq_fastscan_runtime_fixture_*` helpers still produce
   objects named `grouped_v2_*` in the catalog. If any test asserts
   on catalog names (should be none after 388, but worth checking),
   the helper-layer rename is *not* self-contained. Fine for this
   slice given 388 follows, but verify 388 actually closes it.

2. **Linker gap.** Pure helper rename, minimal risk. Least
   concerning packet in the arc for the skipped-tests issue.

### Observation

Right-sized slice. The real question is whether 386+388 together
actually eliminate `grouped_v2` from the tree, which 388's review
has to confirm.
