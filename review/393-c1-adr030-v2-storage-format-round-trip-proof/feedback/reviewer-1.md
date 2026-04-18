## Feedback: Storage Format Round-Trip Proof

Read `test_tqhnsw_turboquant_reloption_round_trip` and
`test_tqhnsw_pq_fastscan_reloption_round_trip` in `src/lib.rs`.

### What's right

- **Each format gets one concise, explicit end-to-end proof.**
  Build → rank inserted row → live-insert → rank again → delete →
  vacuum → assert delete is gone. That's the task-15 landing bar in
  test form. "Both formats pass the reloption round-trip"
  operationalizes what "first-class" means.
- **Both tests use the reloption surface, not an internal helper.**
  The SQL `WITH (storage_format = ...)` path is what users hit; the
  tests exercise that same path rather than an inner-Rust fixture,
  which is correct for a landing proof.
- **Rank-first assertion before the mutation phase.** That's the
  implicit check that the test's initial build is healthy — without
  it, a build regression could silently pass by not ranking the
  inserted row first *either* before or after.
- **Symmetric structure across formats.** Same sequence of
  assertions, only the fixture helper and source-column wiring
  differ. That makes regressions easier to triage: if one format
  passes and the other fails at the same step, the divergence is
  localized to the format-specific code path.

### Concerns

1. **Single-row insert and single-row delete.** The round trip
   exercises the lifecycle but at minimum scale. A lifecycle test
   that inserted N rows, deleted half, vacuumed, then asserted
   recall on the remaining half would catch classes of bugs this
   test can't (e.g., backlink-planning bugs that only manifest on
   dense graphs). Task 15's real-corpus lane is the real answer to
   that, but a slightly denser in-tree round trip would close more
   in the pg-test layer.

2. **No ef_search variation.** Round trip runs at default ef_search.
   A parameter-sweep version of the same assertion would prove the
   reloption path composes cleanly with the ef_search control
   surface — worth doing as a followup if not critical for merge.

3. **Linker gap.** This is the packet whose tests most directly
   encode the task-15 definition of done. Not running them locally
   means the "landing proof" claim is aspirational until the CI
   lane that does run pgrx tests confirms them green.

### Observation

This is the packet that justifies the rest of the branch to a skim
reviewer: "does both formats survive build+insert+vacuum+scan?"
With 390 handling bootstrap and 393 proving lifecycle, the
functional story is complete.
