## Feedback: ADR-030 v2 Grouped Score Helper Context

Read `score_grouped_candidate_context(...)`, `grouped_score_payload_view(...)`, and
the new `GroupedScorePayloadView<'a>` struct in `src/am/scan.rs`.

### What's right

- Helper now takes `GroupedScoreContext<'_>` directly. Every packet from 329 onward
  has progressively widened the helper's input without touching dispatch callsites.
  Net result: when the real scorer lands, the helper body is the only code that
  changes.
- `GroupedScorePayloadView` was added in this packet, not just in the "next slice"
  preview — it's already building the borrowed view that the real scorer will use
  (`search_code`, `binary_words`, `reranktid`, `rerank_code_len`). That's the shape
  you want the scorer to consume.
- `grouped_score_payload_view` validates `binary_words.len() == shape.binary_word_count`
  and `search_code.len() == shape.search_code_len`. Shape check happens before the
  scorer runs, not inside its inner loop. Correct place for it.

### Concerns

1. **Panic on shape mismatch, then `pgrx::error!`.** In
   `score_grouped_candidate_context`, if `grouped_score_payload_view` returns `None`,
   the code `panic!`s. Otherwise it reaches the `pgrx::error!` stub. Two different
   failure paths with different observable behavior. When the real scorer replaces
   the stub body, make sure the shape-mismatch path continues to surface as a
   structured error (not a raw panic), because shape mismatch under real workloads
   would indicate on-disk corruption — on-call engineers need a clear error line,
   not a stack trace from a panic handler.

2. **Unused view in stub.** `let _payload = grouped_score_payload_view(...)` evaluates
   the view, checks shape, then discards it. That's fine as a stub, but the `_payload`
   binding is a readability crutch — when the real scorer lands, this goes away.
   Confirm the scorer packet actually uses every field of the view, so the type is
   load-bearing rather than advisory.

3. **Cold rerank fetch still absent.** `GroupedScorePayloadView` carries
   `rerank_code_len` but the cold rerank tuple at `reranktid` is never read. The
   scorer packet will need to add a cold-tuple fetch. Worth planning: a rerank fetch
   error at the scorer site is the first point where a grouped-v2 query can fail for
   reasons other than "unsupported." The error-path taxonomy matters.

### Wider observation on the 329-333 sequence

Five packets to lay the scorer seam — dispatch, helper stub, shape, context, helper
context. Some reviewers might call that excessive for code that all lands in one
file. But the payoff is visible: the next packet can be "introduce LUT scorer in
one helper body" with no dispatch or cache reshaping. If the real scorer bumps into
an unexpected wrinkle (e.g., needing to carry query-side state), that wrinkle lands
in one place.

The accumulated cost is about ~200 lines of seam code that will not be removed. That
cost is acceptable given what it enables.

### Still-open cross-cutting gaps

Flagged previously at packet 328 and still present:

- `src/am/insert.rs` has no format_version or `GraphStorageFormat` guard. Verified by
  grepping the file for any ADR-030/GroupedV2/format_version mention — zero matches.
- `src/am/vacuum.rs` has no grouped-v2 awareness. Same grep — zero matches.
- `GraphTupleRef::binary_word_count()` on the grouped branch still allocates a Vec
  (`src/am/graph.rs:164`).
- `encode_grouped_pq` still duplicated between `src/am/build.rs:876` and
  `src/bin/approx_score_study.rs:840`.

None block this packet. All block lifting the experimental gate.

### Suggested sequencing from here

If the next priority is the actual scorer, fine — all the seams are in place. But
before grouped-v2 leaves the experimental gate, the insert/vacuum/hot-path
allocation/encoder-duplication items above need resolution. Consider interleaving one
of these with the next scorer packet so they don't all stack at the end.
