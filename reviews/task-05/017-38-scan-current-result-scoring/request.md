# Review Request: Scan Current-Result Scoring

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- The bootstrap linear scan now computes a current-result score as the operator-facing `<#>` value for the current element tuple.
- `amrescan` now caches the index seed alongside the prepared query so scan-local scoring can reuse the quantizer configuration.
- Current-result scoring rebuilds the candidate payload from the element tuple's code bytes plus the representative heap row's persisted `gamma`.
- The shared heap-`gamma` fetch path now falls back to resolving the heap relation from the index relation when manual scan descriptors do not populate `heapRelation`, and it releases heap buffers after fetch.
- Regression coverage now verifies that score validity becomes true on first tuple production, matches the SQL `<#>` result for the representative heap tuple, stays populated while draining duplicate heap TIDs, and clears on exhaustion.

Review focus:
- Whether storing the operator-facing `<#>` value is the right contract for later ordered-scan result state
- Whether the representative-heap-row fallback and buffer-release behavior are safe and coherent for this bootstrap phase
- Whether the current-result score lifecycle is now credible groundwork for future candidate/result ordering state

Questions to answer:
- Should scan-local current-result state store `<#>` values or support-function inner-product values at this stage?
- Is representative-heap-row `gamma` recovery an acceptable temporary bridge until candidate payload state is stored locally?
- Do you see any correctness or resource-lifetime risk in the heap-relation fallback used by manual scan descriptors and tests?
