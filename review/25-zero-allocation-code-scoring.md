# Review Request: Zero-Allocation Code Scoring

Scope:
- `src/lib.rs`
- `src/quant/prod.rs`

What changed:
- Added `ProdQuantizer::score_ip_codes_lite`, which scores raw stored code bytes directly without constructing fake `[gamma][code]` payloads.
- `score_code_inner_product` now routes through that raw-code fast path instead of allocating two temporary vectors per call.
- Quantizer coverage now checks that the new raw-code scorer matches the previous encoded-lite scorer on the same encoded vectors.

Review focus:
- Whether the new raw-code scorer preserves the existing code-to-code semantics while removing allocation from the hot path
- Whether the code-byte split and validation boundaries are correct for the current tqvector layout
- Whether the new quantizer test is enough for this narrow performance-oriented fix

Questions to answer:
- Is `score_ip_codes_lite` the right long-term boundary for build-time and SQL-visible code-to-code scoring?
- Is there any remaining caller that still constructs temporary fake payloads and should move to the same helper?
- Should invalid raw-code length handling stay as `assert!`-based internal invariants here, or is there a better boundary for this path?
