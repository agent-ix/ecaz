# Review Request: Scan Prepared-Query Cache

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` now prepares the quantizer query state for non-empty indexes and stores it in scan-owned memory.
- `amendscan` frees that prepared query state alongside the copied raw query payload.
- Existing rescan-state debug coverage now verifies that non-empty rescans cache prepared query state, while empty-index rescans do not allocate it.

Review focus:
- Whether scan-owned prepared-query caching is the right next boundary for ordered scan execution
- Whether the ownership and teardown of the prepared query object are correct across repeated rescans and `amendscan`
- Whether the added regression coverage is sufficient for this groundwork slice

Questions to answer:
- Is caching `PreparedQuery` at `amrescan` time the right contract for future ordered traversal?
- Is there any stale-state or lifetime risk in replacing the prepared query object on repeated rescans?
- Should any part of this prepared state be stored in PostgreSQL-managed allocations instead of a boxed Rust object at this stage?
