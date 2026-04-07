# Review Request: Scan Metadata Cache

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` now caches the index dimensions, bits, and derived code length in scan-owned opaque state.
- `amgettuple` now uses those cached values instead of rereading the metadata page on every call.
- The rescan debug helpers and regression coverage now assert that cached scan metadata is initialized consistently for both non-empty and empty indexes.

Review focus:
- Whether caching immutable metadata in `TqScanOpaque` is the right boundary for the current scan bootstrap
- Whether the new cached fields are reset and owned correctly across repeated rescans and teardown
- Whether the added regression coverage is sufficient for this narrow hot-path optimization

Questions to answer:
- Is there any stale-state risk in reusing cached `dimensions`, `bits`, and `code_len` after repeated rescans on the same descriptor?
- Should any additional scan metadata be cached now, or is this the right minimum for the current linear scan path?
- Is there a missing regression around rescanning from a non-empty index after prior empty-index state, or vice versa?
