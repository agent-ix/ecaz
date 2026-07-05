## Feedback: ADR-030 v2 Cold Rerank Fetch Seam

Read `GroupedRerankPayload`, `with_grouped_rerank_tuple`, and
`load_grouped_rerank_payload` in `src/am/graph.rs`. Verified: pg-test
`test_grouped_v2_graph_reads_load_cold_rerank_payload` builds a grouped-v2 index,
follows `reranktid`, and decodes the cold tuple end-to-end.

### What's right

- Borrowed (`with_grouped_rerank_tuple`) and owned (`load_grouped_rerank_payload`)
  entry points mirror the hot-tuple read API. Symmetrical reading surface.
- End-to-end pg-test is the load-bearing coverage — builder writes to disk, reader
  reads from disk via reranktid, payload lengths check out. Exercises the page-
  layout contract from packet 314 for real.
- Typed `GroupedRerankPayload` (`tid`, `gamma`, `code`) carries the minimum fields
  the scorer needs. No overreach.

### Concerns

1. **Error path on reranktid pointing to a missing cold tuple.** If a grouped hot
   tuple's `reranktid` is valid but the cold tuple at that TID has been
   overwritten or lost (e.g. half-vacuumed index, truncated heap), what does
   `load_grouped_rerank_payload` do? Panic, ereport, or surface through an Option?
   The happy-path test doesn't tell us. Worth a negative test where the reranktid
   points at a non-existent tuple, and an explicit expected behavior (loud error
   is correct; silent wrong-tuple-decode is wrong).

2. **Layout check on decode.** `load_grouped_rerank_payload` takes a
   `GroupedGraphLayout` and, per the scan-side call at `scan.rs:1160`, the layout
   is reconstructed from the hot payload's widths at call time. That's fine, but
   the payload uses `code.len() != payload.rerank_code_len` only at the scan-side
   `grouped_score_rerank_payload`. If `load_grouped_rerank_payload` decodes a cold
   tuple and silently returns a payload whose `code.len()` doesn't match the
   expected `rerank_code_len`, the scan-side check catches it. But catching at the
   graph-side decode would be stricter. Worth checking the inner code — if decode
   doesn't already validate, the scan-side check is load-bearing.

### Observation

First packet in the sequence where real on-disk cross-tuple traversal is exercised
from a grouped-v2 index, end-to-end. Important milestone. Next step (packet 340)
is to compose hot and cold in the scorer boundary.
