## Feedback: Storage-Format Matrix And Runtime Gate Closure

Read the three reverse-direction mismatch tests at
`src/lib.rs:9183`, `:9209`, `:9237`, the happy-path
`test_tqhnsw_storage_format_switch_reindex_restores_runtime` at
`:9259–9319`, the tightened parity test's resolution-reason
assertions, the named `SCORE_ASSERT_EPSILON = 1e-5` at `:790`,
and the AM unit test
`grouped_binary_traversal_score_gate_requires_pq_fastscan_storage`
at `src/am/scan.rs:6564`.

### What's right

- **Closes blockers 1 and 3 from the `419` readiness
  assessment.** Both reverse-direction mismatch coverage (all
  three AM entry paths) and the REINDEX-restores-runtime
  happy-path are now present. Specifically addresses the
  "turboquant→pq_fastscan only" direction-bias concern I raised
  on `409` and the "no happy-path-after-matching-ALTER" concern
  raised on `409` and `419`.
- **REINDEX happy-path is the whole round-trip, not just a
  compile-check.** `test_tqhnsw_storage_format_switch_reindex_
  restores_runtime` does the full sequence: ALTER reloption →
  REINDEX → assert metadata format actually flipped → assert
  ordered scan still self-ranks → assert INSERT succeeds →
  assert VACUUM removes heap tid. That is the right contract
  phrasing: "after REINDEX, every write path the guardrail was
  blocking now works again." It proves both that REINDEX clears
  the guardrail *and* that the rebuilt index is functionally
  correct — not just compilable. No shortcut.
- **Parity test now asserts resolution reasons, not just
  score equality.** Directly addresses concern #2 from my `411`
  feedback — both lanes producing the same score could have
  masked both hitting `EnvOverride`. Now `default_heap_f32_with_
  build_source_column` vs `env_override` is locked as a
  contract. A stray env leak cannot silently make both lanes
  look like explicit overrides and still pass.
- **`SCORE_ASSERT_EPSILON = 1e-5` named as a constant.** Directly
  addresses concern #2 from my `417` feedback — the epsilon is
  now a single named value instead of scattered magic numbers.
  `1e-5` is also the right order for heap-f32 inner products on
  normalized vectors.
- **Binary-traversal gate unit test is three-case, not
  one-case.** Off for `TurboQuant`, on for `PqFastScan`, off
  when mode changes back. That is the right shape — proves the
  gate *and* that it doesn't latch. A one-case assertion would
  have left the latch hypothesis untested.
- **Turboquant fixture refactored to support both source-less
  and source-backed shapes.** That is the plumbing that made the
  REINDEX happy-path buildable — you can't start with a source-
  backed turboquant and flip to source-backed pq_fastscan unless
  the starting fixture has a source column. Right factoring, not
  a copy.
- **Local `cargo test` runs cited by test name.** The packet
  body names the specific tests that were executed (`cargo test
  test_tqhnsw_storage_format_switch_reindex_restores_runtime --
  --nocapture` and the binary-gate unit test). That is exactly
  the "capture which tests are green" ask from `419` blocker 1,
  at least for these two — more complete than any prior packet
  on the arc.
- **Sandbox vs non-sandbox wrapper status split honestly.**
  "Inside sandbox fails on read-only `cargo pgrx install`;
  outside sandbox passes on current tree." That is the right
  framing — separates environment problems from code problems
  instead of burying the pass behind the sandbox fail.

### Concerns

1. **"Outside sandbox passes on current tree" needs a captured
   artifact.** This is the single most important claim in the
   packet and it is cited without capture. An attached `cargo
   pgrx test pg17` output excerpt showing the test names and
   `ok` counts would convert this from trust-me to evidence.
   Without it, `419` blocker 1 is still open for the broader
   `#[pg_test]` surface — named green only for the two tests
   explicitly cargo-tested, not the rest.
2. **Reverse-direction tests reuse the same panic-message
   assertion shape.** Good for consistency, but if the
   guardrail message ever drifts in one direction (e.g.,
   reverse direction emits a slightly different format name
   order), all four mismatch tests fail together in the same
   way and the failure cluster wouldn't distinguish which
   direction broke. Minor — worth one sentence naming the
   expected message for each direction, or a small message-
   format helper that takes `(from_format, to_format)` and
   both directions call through it.
3. **Happy-path test only covers turboquant→pq_fastscan
   direction.** The reverse happy-path (start pq_fastscan, ALTER
   to turboquant, REINDEX) is not tested. Probably low risk — if
   one direction's REINDEX restoration works, the other almost
   certainly does — but the symmetry would round out the matrix.
   Cheap to add.
4. **No test asserts REINDEX fails if the new format is
   fundamentally incompatible with the existing column type.**
   That is not a current code path — all supported formats
   accept the same tqvector column — but if a future format is
   added that requires a different column type, there is no
   guardrail test that would catch a "REINDEX silently succeeds
   with wrong semantics" regression. Out of scope for this
   packet, worth a follow-up task.
5. **`SCORE_ASSERT_EPSILON = 1e-5` is right for heap-f32 inner
   products on normalized vectors, but may be too tight for
   quantized rerank paths.** If a future test asserts scores on
   a quantized path using this same constant, it could flake on
   last-bit drift. Worth one comment at the constant naming the
   scope it applies to — "exact-source inner-product
   comparisons only."

### Observation

This is the packet `419` specifically asked for. Blocker 1
(captured test execution) is partially closed — named green
tests are now in the packet body, but a broader capture of the
full `#[pg_test]` surface is still needed. Blocker 3 (missing
coverage) is closed for the guardrail surface. The merge-
readiness call from `419` was "85% ready"; with `420` landed,
that is closer to **92%**. What remains is the captured-
execution artifact for the full test surface, landing `416` /
`417` / `418`, and the CI / planner follow-ups that were always
scoped as post-merge.

A reviewer handed this branch today would find it harder to
name a real merge blocker than at any point in the 378–420 arc.
