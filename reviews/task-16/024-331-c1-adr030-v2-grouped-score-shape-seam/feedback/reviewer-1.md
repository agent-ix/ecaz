## Feedback: ADR-030 v2 Grouped Score Shape Seam

Read `GroupedScoreShape`, `GroupedScoreShape::from_scan_graph_storage(...)`, and
`GroupedScoreCall<'a>` in `src/am/scan.rs`.

### What's right

- `GroupedScoreShape` as a small, copyable, value-typed struct is exactly the shape a
  LUT scorer will want to carry into its inner loop. No indirection, no allocation.
- `from_scan_graph_storage` returns `Option`: `Some` only for GroupedV2, `None` for
  ScalarV1. That prevents a silent dispatch of grouped scoring on a scalar index.
- `GroupedScoreCall<'a>` bundles shape and input. That's the right minimum surface
  for the helper — shape for the loop bounds, input for the bytes.

### Concerns

1. **`rerank_code_len` is carried but not yet validated.** The shape propagates
   `rerank_code_len` from `GroupedGraphLayout`. Packet 333's helper uses it in
   `GroupedScorePayloadView` with no validation yet against the cold rerank tuple
   actually fetched. That's fine for now, but the first real scorer packet that
   reads rerank bytes must check it. File a tracking note in the scorer packet to
   avoid forgetting.

2. **`binary_word_count` drift risk.** The shape says "N binary words expected." The
   cached grouped payload carries whatever the hot tuple encoded. Packet 333's
   `grouped_score_payload_view` does a length check. Good. But `GraphStorageDescriptor`
   is built from metadata at scan-open; the hot tuple was encoded at build time. A
   build-time vs scan-time metadata mismatch (not expected, but possible under
   partial upgrade scenarios) would hit the shape check. Worth a test that
   explicitly constructs a shape/input mismatch and asserts the scorer errors loudly
   rather than silently truncating.

### Observation

Making dispatch metadata-aware without widening helper inputs past `GroupedScoreCall`
was the right size for this packet. The structure scales: packet 332 then adds one
outer `GroupedScoreContext` without reshuffling shape/input.
