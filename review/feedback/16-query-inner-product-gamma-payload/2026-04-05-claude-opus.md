# Feedback: `tqvector_query_inner_product` Rebuilds Full Payload

Request:
- `review/16-query-inner-product-gamma-payload.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is rebuilding the payload inline the right implementation boundary?

**Yes, for now.** The inline reconstruction at `score_query_inner_product` (line 274-291) is straightforward:

```rust
let mut payload = Vec::with_capacity(4 + codes.len());
payload.extend_from_slice(&gamma.to_le_bytes());
payload.extend_from_slice(codes);
Ok(quantizer.score_ip_encoded(&prepared, &payload))
```

This allocates one `Vec<u8>` per SQL function call. For row-by-row SQL scoring (`SELECT ... ORDER BY embedding <#> query`), this is acceptable — the function call overhead dominates.

**However, when this is used inside `amgettuple` for graph traversal (future),** this pattern will be too expensive. At that point, `score_ip_encoded` should accept `(gamma, code_slice)` directly without requiring a contiguous payload. Consider adding a `ProdQuantizer::score_ip_from_parts(prepared, gamma, mse_packed, qjl_packed)` method now, or at least noting this as a planned optimization for the graph traversal slice.

A shared helper is not needed at this stage because the only two call sites have different input shapes:
- SQL function: unpacks `(dim, bits, seed, gamma, codes)` from the varlena
- Future scan: reads `(gamma, codes)` from an in-page element tuple

These have different enough entry points that a shared helper would just be another indirection.

### Is there any missing coverage around invalid payloads or negative wrapper?

**One gap.** The test verifies that `tqvector_query_inner_product` matches `ProdQuantizer::score_ip_encoded` for a well-formed candidate. But there's no test for:

1. **Dimension mismatch between candidate and query.** `score_query_inner_product` checks this at line 277 and returns an error. A regression test that calls `tqvector_query_inner_product` with a 4-dim candidate and an 8-dim query, expecting a dimension mismatch error, would cover this.

2. **`tqvector_negative_query_inner_product` correctness.** The negative wrapper (line 299) simply negates. A test that verifies `negative_query_ip = -query_ip` for a known input would confirm the wrapper works end-to-end. This is trivial but documents the contract.

Both are low priority since the logic is minimal, but they'd close coverage gaps in the SQL-surface tests.

### Does this change expose any inconsistency with the index tuple layout?

**Yes — worth calling out explicitly.** The SQL function reconstructs the payload as `[gamma][codes]` from a varlena datum where the wire format is `[dim][bits][seed][gamma][codes]`. The index element tuple stores only `codes` (no gamma) because the gamma is baked into the `tqvector` varlena stored on the heap.

When the scan eventually scores candidate tuples from the index (not the heap), it will need to either:
1. Fetch the full `tqvector` datum from the heap to get gamma (requires heap access per candidate — expensive)
2. Store gamma in the element tuple alongside the code (requires a page layout change)
3. Use `score_ip_encoded_lite` (code-to-code, no gamma) for the index-side scoring

Option 3 matches the current `score_ip_encoded_lite` which ignores gamma. For the HNSW graph traversal, this would mean the reranking step needs to use full heap tuples to get gamma-corrected scores, while the graph search uses gamma-free scores.

This is a design decision for the scan slice, not a bug. But it should be documented now so the scan implementation doesn't accidentally assume gamma is available in element tuples.

## Additional Findings

No correctness issues found. The payload reconstruction is correct and matches the `ProdQuantizer` contract. The `Vec` allocation is fine for the SQL-function use case.
