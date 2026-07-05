## Feedback: ADR-030 v2 Grouped Score Context Seam

Read `GroupedScoreContext<'a>` and `grouped_score_context_from_scan_state(...)` in
`src/am/scan.rs`, plus the two new tests:
- `grouped_score_context_uses_scan_shape_and_cached_payloads`
- `grouped_score_context_requires_grouped_scan_storage`

### What's right

- `GroupedScoreContext` adds `element_tid` to the existing `GroupedScoreCall`. That's
  the minimum extension. The scorer needs to know which element it is scoring (for
  logging/rerank identity); carrying the tid through dispatch is cheaper than
  threading it separately.
- `grouped_score_context_from_scan_state` returns `Option`: it short-circuits if the
  scan storage is ScalarV1, or if the cached element has no grouped hot payloads.
  Correct composition of the two guards.
- Second test (`grouped_score_context_requires_grouped_scan_storage`) is the right
  negative test — it verifies that scalar scan storage cannot produce a grouped
  context.

### Concern: panic in dispatch

`candidate_score_dispatch` calls this helper and `unwrap_or_else(|| panic!(...))` if
it returns `None`. In practice this can only fire under these conditions:

- `LoadedElementState::ExactUnavailable` (can currently only arise for grouped-v2
  live tuples)
- `scan_graph_storage` is ScalarV1 (shouldn't happen for a grouped-v2 live tuple)
  **or** `element.grouped_score_input()` returns `None` (shouldn't happen for a
  grouped-v2 live tuple)

So the panic is an "invariants violated, we shouldn't be here" path. That's
acceptable, but:

1. The panic message says "requires grouped score context" — that's not very
   actionable for an on-call engineer. Consider making it an `ereport` with the
   concrete state: which arm of `LoadedElementState`, which variant of
   `GraphStorageDescriptor`, whether `grouped_score_input` was `None`. The failure is
   rare enough that the extra message cost is irrelevant.

2. Long-term, as I noted on packet 329, the cleaner shape is to branch dispatch on
   `GraphStorageDescriptor`, not on `LoadedElementState`. Then this panic becomes
   impossible because the types align.

### Observation

Moving from "carry inputs inline in dispatch" to "build one typed context in a
helper" is a small structural change that pays off: packet 333 then flips the
dispatch to carry `GroupedScoreContext` directly, and the first real scorer packet
can build against this context without further reshaping.
