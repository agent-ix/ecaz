# Review Request: Linear Scan Exhaustion and Direction Guards

Feedback dir:
- `review/feedback/17-linear-scan-exhaustion-and-direction-guards/`

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added explicit regression coverage for the current linear scan bootstrap after it drains all tuples.
- The new coverage verifies that once the scan is exhausted, repeated `amgettuple` calls continue to return `false`.
- Added explicit regression coverage that `amgettuple` still errors on backward scan direction even after a valid `amrescan`.

Review focus:
- Whether the current exhausted-scan contract is stable enough for later ordered-search work
- Whether backward-direction rejection is enforced at the right state-machine boundary
- Whether the new debug helpers and tests are sufficient for this narrow scan-lifecycle slice

Questions to answer:
- Is there any missing regression around rescanning after exhaustion, not just after partial duplicate drain?
- Should exhausted-scan behavior assert anything about retained `xs_heaptid` state, or is `false`-only coverage enough for this stage?
- Is there any reason to soften the backward-direction error before ordered traversal exists, or is strict rejection the correct current contract?
