## Feedback: ADR-030 v2 Grouped Payload View

This is what packet 333 previewed; the view is now its own packet with dedicated
tests.

### What's right

- `grouped_score_payload_view_rejects_shape_mismatch` is exactly the negative test I
  asked for on packet 331. Shape/input mismatch now fails loudly rather than silently
  truncating.
- The view validates both `binary_words.len()` and `search_code.len()` against the
  metadata-derived shape before producing the view. Validation is outside the scorer
  inner loop, in the right place.
- Both positive and negative tests land together. Good discipline.

### Observation

The validation is duplicated in a thin sense: `GraphStorageDescriptor::from_metadata`
validates metadata shape at scan open; `grouped_score_payload_view` re-validates
payload-vs-shape at score time. They're different invariants (metadata self-
consistency vs. tuple-vs-metadata consistency), so keeping both is correct. But it
means a tuple corruption produces a shape-mismatch error at score time with no easy
pointer back to which tuple. Worth including `element_tid` in the error if the view
ever starts surfacing one — today the `None` is silent and the caller panics one
layer up.

### No significant concerns

Seam is the right size. Good setup for the rerank payload work in packet 340.
