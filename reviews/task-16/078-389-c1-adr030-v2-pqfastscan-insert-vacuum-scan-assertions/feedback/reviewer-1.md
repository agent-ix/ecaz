## Feedback: PqFastScan Insert+Vacuum Scan Assertions

Read the extended `test_tqhnsw_insert_appends_to_built_pq_fastscan_index`
and `test_tqhnsw_vacuum_pass2_unlinks_pq_fastscan_refs` in `src/lib.rs`.

### What's right

- **Moves from structural to behavioral assertions.** Structural
  tuple-shape tests prove the bytes were written. Ordered-scan
  assertions prove the bytes were *correct enough to surface the
  right result*. That's the jump from "no regression in layout" to
  "no regression in user-visible behavior."
- **Insert test asserts ordered-scan rank for the inserted row's own
  embedding.** That's the right self-check: if the row doesn't win
  ordered scan against its own query vector, the search code or the
  graph edges were persisted wrong. It's a near-zero-noise assertion.
- **Vacuum test does before/after ordered scan.** Proving that a row
  was rank 1 before delete *and* no longer appears after
  delete+vacuum catches regressions in either direction — incomplete
  tombstoning would leave the row visible; broken graph repair
  might still return it via stale neighbor links.
- **Deterministic reconstruction of the deleted row's embedding.**
  Using the runtime-fixture formula rather than recording the
  embedding separately keeps the test hermetic. Nothing to drift
  out of sync.

### Concerns

1. **Only tests layer-0 (implicitly).** Neither assertion exercises
   the upper-layer graph — an upper-layer repair bug could still
   pass these. The structural tests earlier in the arc may cover
   that, but the behavioral proof here doesn't. Worth adding a
   higher-M variant where the deleted node was an upper-layer
   entry/greedy-descent target.

2. **The "deleted row no longer appears" assertion is binary.**
   It's a strong correctness check but doesn't verify ranking
   *quality* after the delete — a vacuum that silently degraded
   recall by 40% while still not returning the deleted row would
   pass. That's fine for this slice, but the real-corpus recall
   lane (packet 398+) is the quality gate.

3. **Linker gap.** Behavioral tests that don't run in the local
   environment are worth very little on their own. The new
   assertions become load-bearing only in whatever CI lane runs
   pgrx tests — confirm that lane is green before merge.

### Observation

Right kind of proof to add before landing. Structural tests are
necessary but not sufficient; this packet moves the bar to "the
tuples on disk produce the right scan behavior," which is what task
15 actually promises.
