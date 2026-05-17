## Feedback: PqFastScan Default Rerank Parity

Read `test_pq_fastscan_default_rerank_matches_explicit_heap` at
`src/lib.rs:4452+`, and cross-referenced to packet `404`'s
`test_pq_fastscan_default_source_rerank_emits_heap_scores` at
:4233+ and the sibling explicit-`heap_f32` test at :4274+.

### What's right

- **Fills the exact gap packet `404`'s feedback named.** The
  earlier proof was "default emits heap-shaped scores," and this
  packet upgrades it to "default is score-identical to explicit
  `heap_f32`." That is the proof that actually protects merge
  against a future where the two lanes silently diverge — e.g.,
  if someone adds a rerank-path branch that only fires on env
  override.
- **Parity is phrased at the operator-facing surface.** Same
  ordered scan output, same emitted score, same comparison score,
  same approximate-rank sequence. Not "both helpers route to the
  same internal selector" — a contract test phrased on the
  observable output, which is the right level to lock a parity
  claim.
- **Test-only, zero runtime-diff.** Correctly scoped. A parity
  proof that also nudged the runtime would muddy which direction
  the safety comes from.
- **Complements `404` rather than supersedes.** `404`'s ground
  truth is `-dot_product(query, source(id))`; this test's ground
  truth is "the other lane." Together they pin both ends: the
  default lane scores match external truth AND match explicit
  `heap_f32`. Removing either leaves a hole.

### Concerns

1. **Parity is asserted on one fixture shape only.** Same small
   runtime fixture as `404`, same query. If `heap_f32` has a
   fixture-dependent divergence (e.g., triggers only at certain
   layout word-counts, or on ties), this test cannot see it.
   Cheap follow-up: parametrize over two fixture shapes — one
   small (current), one at the production `m=16, 10k` scale.
   That would turn parity from "identical on this one case" into
   "identical on the shapes we care about."
2. **No assertion on *which* lane chose heap.** Both lanes
   produce identical scores, but the test does not inspect the
   rerank-mode resolution from packet `410`'s debug helper to
   confirm one lane chose
   `DefaultHeapF32WithBuildSourceColumn` and the other chose
   `EnvOverride`. Without that, a subtle regression where both
   lanes hit `EnvOverride` (e.g., a stray env leak) would pass
   silently. Add a two-line helper readback to lock the
   resolution-reason contract too.
3. **No ties / degenerate-ordering case.** The test compares
   approximate-rank sequences, so if rerank ever produces exact
   ties both lanes would need deterministic tiebreaking to keep
   matching. Probably fine — heap_f32 is deterministic in both
   lanes — but worth one sentence either confirming no ties on
   this fixture, or adding a short tie-producing case.
4. **Test has never executed pre-`415`.** Same linker gap as the
   rest of the arc at the time this packet was written. Packet
   `415` now makes plain `cargo test` a real checkpoint, so this
   test has a path to actually run — but it needs to be captured
   as green in a landing-proof artifact, not claimed by source
   inspection.

### Observation

Exactly the shape of safety-check packet `404` needed. The
parity-to-the-operator-surface assertion is a higher bar than
"both paths reach the same function," and it is the bar that
will survive refactors. Worth extending the same pattern to
packet `408`'s traversal-mode fallback (prove the fallback path
is score-identical to explicit `pq`) once another recall
investigation asks for it.
