# Request: Skip Stale Bootstrap Candidates Before Linear Fallback

Commit: `f594ff7`

Summary:
- Bootstrap result production now keeps consuming queued bootstrap candidates until one actually materializes, instead of falling through to linear scan after the first dead/non-materializable candidate.
- The new helper is pure and unit-tested for both “skip one and succeed later” and “everything fails” cases.
- Runtime behavior is otherwise unchanged: once no bootstrap candidate materializes, execution still falls back to the existing linear scan path.

Files:
- `src/am/scan.rs`

Why this matters:
- Before this change, one stale or deleted bootstrap candidate could prematurely abandon the beam-led frontier even if other live bootstrap candidates were still queued.
- That behavior weakened the current graph-search path exactly where it should be strongest: consuming score-ordered frontier candidates before reverting to linear fallback.
- This slice makes the current staged execution more robust without changing the broader planner-disabled scan contract.

Review focus:
- Whether the new helper preserves the intended ordering semantics while skipping non-materializable candidates
- Whether the raw-pointer split-borrow at the helper call site has a sound non-aliasing contract
- Whether any bootstrap candidate failure cases should still terminate immediately instead of continuing to later queued candidates
