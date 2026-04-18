## Feedback: ADR-030 v2 Gated Grouped Scan Runtime

This is the packet. Grouped-v2 scans now actually execute behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`. Verified:

- `validate_runtime_scan_format` at `scan.rs:674` accepts grouped-v2 only when
  `experimental_grouped_v2_scan_enabled()` returns true.
- `score_grouped_candidate_context` at `scan.rs:1358` now returns a real
  approximate score via `score_grouped_search_code_from_scan_state` → shared
  `grouped_pq_score_f32`.
- `GraphTupleRef::binary_word_count()` at `graph.rs:199` now calls
  `tuple.binary_word_count()` directly on both branches. The Vec allocation I
  flagged at packet 324/333/343 is closed.
- The env-var gate is single-purpose: presence-only check
  (`std::env::var_os(...).is_some()`), no value parsing. Good default-deny
  posture.

### What's right

- Two separate gates (build, scan) for v2. An operator with only the scan gate
  set cannot build v2 indexes; an operator with only the build gate set cannot
  execute them. That's a good safety property — someone experimenting with v2
  has to explicitly opt in at both ends.
- Per-rescan LUT preparation. `amrescan` loads the persisted codebooks once and
  builds the LUT once per query. The LUT then rides on the prepared-query
  lifetime. No per-candidate LUT rebuild.
- Approximate path no longer does cold-tuple IO. That was my concern on
  packets 340/342 — every "unsupported" call was fetching cold rerank bytes.
  Now the approximate path skips cold fetch entirely (line 1363:
  `let _ = index_relation`). Good. Exact rerank IO is confined to the
  comparison-score helper (`grouped_candidate_rerank_comparison_score`), which
  is only reached when the comparison output is needed.

### Concerns

1. **`candidate_score_dispatch` calls `grouped_score_context_from_scan_state`
   twice.** At lines 1342 and 1345 in the same arm — once as an `if` guard
   (`.is_some()`) and once inside the arm (`.unwrap_or_else(...)`). Duplicate
   construction of the same value. The Rust pattern for this is `if let
   Some(ctx) = ...`. Concretely:

   ```rust
   LoadedElementState::ExactUnavailable | LoadedElementState::None
       if let Some(ctx) = grouped_score_context_from_scan_state(scan_graph_storage, element) =>
   {
       CandidateScoreDispatch::Grouped(ctx)
   }
   ```

   Or pre-compute the Option once before the match. Not a correctness issue but
   worth fixing while the path is young.

2. **Match arm extended to include `LoadedElementState::None`.** Previously
   only `ExactUnavailable` routed to grouped. Now `None` also routes to grouped
   when grouped payloads are present. That's consistent with my packet 329
   feedback about preferring format-driven dispatch over state-driven dispatch
   — but it *half-solves* it: the branch now keys off "grouped payload
   present" (via `is_some()`) rather than pure descriptor. That's better than
   before; the cleanest shape is still descriptor-driven at the outer match
   level, with loaded state only consulted in the Exact arm. Flag for the next
   refactor window.

3. **Scalar scans still walk the same `candidate_score_dispatch`.** For scalar
   indexes the `if` guard always fails (no grouped payloads in cache). That's
   correct but means every scalar candidate scoring call now evaluates a
   `grouped_score_context_from_scan_state` construction that will always
   return None for scalar. Should be cheap (it checks `scan_graph_storage` is
   GroupedV2 first), but worth confirming — scalar-path regressions from
   overhead in the candidate-score inner loop are exactly the failure mode
   that hurts latency without tripping tests.

4. **Gate-lift readiness signals.** Packet 321's build gate message said
   "experimental, not covered by upgrade guarantees." What does the scan gate
   log on activation? If nothing, there's no audit trail of when a server
   started accepting grouped-v2 queries. Consider a one-time INFO log on first
   successful `validate_runtime_scan_format` for a GroupedV2 descriptor.

### Measurements

None in this packet, as stated. The next packet (346) wires a comparison
measurement surface — that's the right next step. But before anything lifts
either gate by default, we need: (a) end-to-end recall on a real corpus, (b)
latency comparison against scalar-v1 with binary prefilter. The evidence
packets 346-349 are shaping this up, but none of them report the actual
numbers yet — they build the surfaces.

### Observation

This is the single biggest behavioral change in the v2 lane so far. Everything
before 345 was plumbing; 345 is execution. Worth a follow-up packet soon that
actually reports one measurement: "recall@10 on a known corpus at ef_search=X
is Y," as a first data point to calibrate the rerank-window decisions that
347-349 are building toward.
