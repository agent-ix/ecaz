## Feedback: First-Class Storage Format Docs And Tests

Read the README "Choosing A Format" section and
`test_tqhnsw_turboquant_storage_format_build_writes_scalar_pages` in
`src/lib.rs`.

### What's right

- **README finally tells users how to choose a format.** Task 15
  specifically lists "README section on choosing a format" as a
  definition-of-done item. The rule-of-thumb language ("TurboQuant
  for small/medium indexes; PqFastScan once measured") matches the
  task-15 framing rather than overselling PqFastScan.
- **REINDEX migration note is explicit.** That closes the ADR-032
  migration contract — users know format switches are not automatic
  and are told upfront, not after they try `ALTER INDEX SET
  (storage_format=...)` and get unclear behavior.
- **Explicit `turboquant` reloption test proves the scalar path
  still works via the new selection surface.** Before this, the
  scalar path only got exercised via "no reloption set = default."
  An explicit `WITH (storage_format='turboquant')` build is the
  matching assertion that the *chosen* turboquant path also yields
  the right on-disk layout. That's symmetric proof with
  `pq_fastscan`.
- **Test verifies absence of grouped tuples.** The assertions check
  scalar element + neighbor tuples are present *and* grouped hot /
  rerank / codebook tuples are absent. That's the right way to
  write a build-layout test — presence-only assertions can pass
  even if extra tuples land on disk.

### Concerns

1. **README guidance is qualitative.** The "once measured" phrasing
   is honest given the current measurement state, but a user
   choosing today still has no numeric guidance. Packet 393's
   round-trip proofs + the real-corpus lane from 398–400 will
   eventually let us replace "once measured" with an actual
   recall/latency comparison. Worth tracking as a README followup.

2. **No ALTER INDEX enforcement test.** README says "switching
   format requires REINDEX" but nothing actively verifies that
   `ALTER INDEX ... SET (storage_format=...)` is either rejected
   or a no-op. Easy to add, protects against a future regression.

3. **Linker gap.** The new turboquant build/layout test did not
   run locally. Structural build-layout tests are the lowest-risk
   category to skip, but they're also the most direct proof that
   the reloption selection surface does what the README claims.

### Observation

Small but load-bearing packet for task 15's definition of done.
Without README + explicit turboquant test, the branch would be
technically feature-complete but landing-incomplete.
