# Review Request: Tail-Page Reuse And Rollover

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- Live inserts no longer always require a fresh page.
- The AM now reuses the current tail data page when it has room for the neighbor+element pair.
- If the tail page cannot fit the pair, insert allocates a new data page and appends there.

Review focus:
- Free-space and tuple-fit accounting
- Page-selection logic for append targets
- Whether page rollover could leave metadata or tuple linkage inconsistent
- Whether current tests actually prove the intended boundary

Questions to answer:
- Is there an off-by-one or packing/alignment bug in the fit check?
- Could rollover choose the wrong page or mis-handle the first tuple on a new page?
- Is there a better minimal test for page-boundary behavior?
