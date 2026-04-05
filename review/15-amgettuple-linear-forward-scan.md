# Review Request: `amgettuple` Linear Forward Scan Bootstrap

Scope:
- `src/am/mod.rs`
- `src/lib.rs`
- `spec/adr/ADR-008-linear-scan-bootstrap.md`
- `spec/adr/ADR-009-linear-scan-duplicate-heaptids.md`

What changed:
- `amgettuple` no longer errors immediately for every non-empty index.
- After a valid `amrescan`, it now performs a forward-only linear walk over data pages.
- The scan keeps page/offset cursor state in scan opaque memory and advances it across repeated `amgettuple` calls.
- Each live element tuple now returns every stored heap TID before the scan advances to the next
  element tuple.
- Duplicate heap-TID progress is also kept in scan opaque memory and is reset by `amrescan`.
- Empty-index scans still return `false`, planner behavior is still unchanged, and graph traversal remains unimplemented.

Review focus:
- Safety of the page/offset cursor state and pending duplicate heap-TID state across repeated
  `amgettuple` calls and `amrescan`
- Correctness of the temporary linear tuple-production contract
- Whether the forward-only restriction and duplicate-draining behavior are acceptable for this
  narrow stage

Questions to answer:
- Is the current cursor advancement logic safe against skipped tuples, page boundaries, exhausted
  scans, and rescans after partially draining a duplicate-coalesced tuple?
- Is storing pending duplicate heap TIDs in scan opaque state the right current boundary, or is
  there a stale-state risk across repeated rescans and teardown?
- Is there a missing regression test around repeated `amgettuple` exhaustion, backward scan
  rejection, or duplicate-heavy scans that span page boundaries?
