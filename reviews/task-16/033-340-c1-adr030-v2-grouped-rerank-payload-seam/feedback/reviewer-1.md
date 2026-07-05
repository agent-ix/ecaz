## Feedback: ADR-030 v2 Grouped Rerank Payload Seam

Read `GroupedScoreRerankPayload`, `grouped_score_rerank_payload`,
`load_grouped_score_rerank_payload`, and the updated `score_grouped_candidate_context`
in `src/am/scan.rs`.

### What's right

- Three narrow helpers, one per concern: shape-match composition
  (`grouped_score_rerank_payload`), disk load + compose
  (`load_grouped_score_rerank_payload`), and scorer dispatch. Tight separation of
  concerns.
- `grouped_score_rerank_payload` rejects both `tid` mismatch and `code.len()`
  mismatch. That catches the two ways a cold-tuple fetch could return the wrong
  bytes for a given reranktid.
- `load_grouped_score_rerank_payload` synthesizes a `GroupedGraphLayout` from the
  hot payload widths before calling into the graph loader, so shape is consistent
  from hot tuple through cold fetch. Good composition.
- Stub helper now exercises every boundary it will need: hot shape validation, cold
  fetch, cold/hot composition, all before the gate error.

### Concerns

1. **`reranktid` mismatch check.** `rerank.tid != payload.reranktid` returns `None`,
   which propagates up through `load_grouped_score_rerank_payload` and then causes
   the panic in `score_grouped_candidate_context`. That's fine defensively, but if
   a reranktid mismatch ever actually happens, the panic message is generic:
   "grouped score helper requires metadata-aligned grouped payload view." Much more
   useful would be "reranktid mismatch: expected X, got Y." Before the gate lifts,
   the error taxonomy on these None-returning helpers should be upgraded from
   "Option-erased" to structured error.

2. **Mismatch test covers shape-mismatch width, not wrong-tid.** `grouped_score_rerank_
   payload_rejects_mismatched_cold_payload` is asserted in code at lines 3805-3838
   — confirm it exercises both the `tid != reranktid` branch *and* the
   `code.len() != rerank_code_len` branch. If it tests only width, the tid branch
   is untested. Worth checking (I didn't read the test bodies).

3. **Stub cost growing.** Every grouped call now does: disk read for cold tuple →
   composition check → panic/error. That's ~I/O per unsupported call. Today that
   only matters in tests (the runtime gate is upstream), but if the gate is ever
   lowered without the scorer being real, this path burns I/O for rejected queries.
   Fine for now, but plan to remove the I/O from the stub path once the real
   scorer lands.

### Observation

By the time the stub reaches `pgrx::error!`, it has already done every step the real
scorer will do except the actual approximate scoring. That's exactly the shape you
want — the scorer packet is now "add the score computation" rather than "wire the
plumbing and add score computation."
