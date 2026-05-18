## Feedback: ADR-030 v2 Grouped Score-Input Seam

Read `GroupedScoreInput<'a>` and `CachedGraphElement::grouped_score_input()` in
`src/am/scan.rs`, plus:
- `grouped_score_input_uses_cached_grouped_hot_payloads`
- `cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads` (now
  strengthened to assert `grouped_score_input() == None`)

### What's right

- `GroupedScoreInput<'a>` carries exactly the three inputs the grouped scorer needs:
  `reranktid`, `search_code`, `binary_words`. Not the whole cache entry. That's the
  right minimum surface.
- Borrowing (`<'a>`) rather than copying. The scorer can operate on cached bytes
  without reallocation.
- `grouped_score_input()` returns `Option` — `Some` only when grouped hot payloads
  are present. Scalar cache entries return `None`. The scorer dispatch can key off
  this directly.

### The important architectural payoff

With this seam in place, the next scorer packet is a *local* change to the candidate-
score dispatch, not a reshape. That's what the incremental slicing strategy from 310
onward has been building toward. Payoff realized here.

### Concerns

1. **`binary_words` in `GroupedScoreInput`.** Is this the binary sidecar from ADR-031
   or the grouped search code re-interpreted? The field name is the same as what the
   ADR-031 binary prefilter consumes. If the intent is to let the grouped scorer
   reuse the ADR-031 binary prefilter result, that's the right design; make sure it
   is documented in the ADR so there's no confusion. If it's a separate concept, the
   field name will mislead.

2. **Lifetime extension.** The `'a` on `GroupedScoreInput` ties the view to the cache
   entry. When the scorer returns a score, does that score carry any references
   back into the cache, or is it a plain `f32`? A plain `f32` result will avoid
   lifetime issues downstream; flagging in case it's not obvious yet.

### Observation

Four packets (324-328) to land read-side plumbing without enabling runtime is a lot,
and it's exactly right. Each packet is independently testable, each advances the
architecture, none of them enable grouped scoring. The next packet can put a narrow,
testable scorer at the candidate-score dispatch point.

### What still blocks lifting the gate

None of the 324-328 packets address:
- Insert path. `src/am/insert.rs` has no format-version guard. A v2 index would today
  accept a scalar insert via that path if `build_source_column` were ever unset.
- Vacuum path. `src/am/vacuum.rs` decodes via `TqElementTuple::decode`. A vacuum of
  a v2 index would mis-decode.
- Rerank fetch. The cold-tuple read path is still unwired.
- End-to-end recall measurement. Packet 311 gave spearman on a study harness. There
  is no end-to-end recall number on real data through the full grouped-v2 pipeline
  yet. That is the headline measurement the gate-lifting decision will rest on.

These are next-slice concerns, not packet-328 concerns. But they should be named now
so they don't get compressed out of the plan.
