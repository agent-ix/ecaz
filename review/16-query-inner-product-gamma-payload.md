# Review Request: `tqvector_query_inner_product` Rebuilds Full Payload

Scope:
- `src/lib.rs`

What changed:
- `tqvector_query_inner_product` no longer passes only the stored `code_bytes` slice into `ProdQuantizer::score_ip_encoded`.
- It now rebuilds the scorer input from the candidate's persisted `gamma` plus `code_bytes`, matching the quantizer payload contract used elsewhere.
- A direct regression test now checks that the SQL-facing query scorer matches `ProdQuantizer::score_ip_encoded` on a full packed payload.

Review focus:
- Whether reconstructing `[gamma][code_bytes]` at the SQL function boundary is the right contract for the current prepared-query scorer
- Whether the new helper boundary keeps error handling and payload ownership simple enough for later scan reuse
- Whether the regression coverage is sufficient for this narrow correctness fix

Questions to answer:
- Is rebuilding the payload inline from unpacked candidate fields the right implementation boundary, or should there be a shared quantizer helper for this shape?
- Is there any missing coverage around invalid candidate payloads or negative-query wrapper behavior after this fix?
- Does this change expose any inconsistency between SQL row-by-row scoring and the current `tqhnsw` index tuple layout that should be called out explicitly in a future slice?
