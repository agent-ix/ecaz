# Review Request: Empty-Index `amgettuple` No-Op

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amgettuple` still enforces the current scan lifecycle gates.
- After a valid `amrescan`, it now returns `false` instead of erroring when the index metadata still indicates an empty index.
- Non-empty scan execution remains blocked with the existing `"not implemented yet"` error.

Review focus:
- Whether the empty-index fast-path is semantically safe for the current planner-disabled scan state
- Whether the empty-index check is placed at the right point in the `amgettuple` state machine
- Whether the added regression coverage is sufficient for this narrow behavior change

Questions to answer:
- Is returning `false` for empty indexes the right current contract, or should empty scans still error until tuple production exists?
- Does checking metadata in `amgettuple` introduce any lifecycle or locking issue in the current implementation?
- Is there a missing regression test around repeated empty scans or repeated rescans on an empty index?
