# Review Request: `amgettuple` Linear Forward Scan Bootstrap

Scope:
- `src/am/mod.rs`
- `src/lib.rs`
- `spec/adr/ADR-008-linear-scan-bootstrap.md`

What changed:
- `amgettuple` no longer errors immediately for every non-empty index.
- After a valid `amrescan`, it now performs a forward-only linear walk over data pages.
- The scan keeps page/offset cursor state in scan opaque memory and advances it across repeated `amgettuple` calls.
- For this bootstrap slice, each live element tuple returns only its first stored heap TID.
- Empty-index scans still return `false`, planner behavior is still unchanged, and graph traversal remains unimplemented.

Review focus:
- Safety of the page/offset cursor state across repeated `amgettuple` calls and `amrescan`
- Correctness of the temporary linear tuple-production contract
- Whether the forward-only restriction and first-heap-TID behavior are acceptable for this narrow stage

Questions to answer:
- Is the current cursor advancement logic safe against skipped tuples, page boundaries, and exhausted scans?
- Is returning only the first heap TID from a duplicate-coalesced element tuple acceptable for this bootstrap slice, or does that create a correctness issue that should be fixed before graph traversal work?
- Is there a missing regression test around repeated `amgettuple` exhaustion, backward scan rejection, or rescan-after-partial-consumption?
