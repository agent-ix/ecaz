# Review Request: Benign No-Op Vacuum Callbacks

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `ambulkdelete` and `amvacuumcleanup` no longer hard-error for `tqhnsw`.
- Both callbacks now return a valid `IndexBulkDeleteResult` without reclaiming tuples or mutating index pages.
- Reported stats currently include main-fork page count and an exact count of element tuples.

Review focus:
- Whether this no-op vacuum behavior is semantically safe for the current live-insert/build state
- Accuracy and safety of the returned `IndexBulkDeleteResult`
- Any hidden assumptions around dead tuples, deleted heap rows, or executor/vacuum expectations

Questions to answer:
- Is reporting exact element-tuple count with zero removals the right current contract?
- Is there any vacuum caller expectation we still violate by leaving stale tuples untouched?
- Are there missing tests around repeated vacuum calls or fully empty indexes?
