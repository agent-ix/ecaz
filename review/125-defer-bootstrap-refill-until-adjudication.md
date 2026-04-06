# Request: Defer Bootstrap Refill Until Successful Candidate Adjudication

Commit: `1c96e9b`

Summary:
- Bootstrap result production now consumes visible bootstrap candidates in score order, tries to materialize each one, and only refills from the consumed source after a candidate actually materializes.
- Dead or stale candidates are still skipped, but they no longer trigger neighbor refill ahead of already-visible later candidates.
- A new pure regression test pins the intended ordering: `A` may fail, `B` may succeed, and refill only runs for `B`.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous consume-and-refill path could introduce newly discovered candidates before the executor had exhausted already-visible frontier candidates.
- That weakened the current staged graph-search path by letting refill semantics perturb visible emission order in the middle of bootstrap adjudication.
- This slice keeps the runtime contract tighter without pretending the scheduler is already authoritative for the entire visible frontier.

Review focus:
- Whether delaying refill until successful materialization matches the intended current bootstrap ordering semantics
- Whether any dead-candidate cases should still refill immediately instead of being treated as consumed-and-discarded
- Whether the new helper keeps the raw-pointer split-borrow contract sound at the runtime call site
