# Review Request: Scan Current-Result Lifecycle

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added regression coverage for the new scan-local current-result slot across duplicate draining, exhaustion, and rescan.
- The coverage verifies that duplicate heap TIDs from one element keep the same current-result tuple pointer until the scan advances.
- The coverage also verifies that exhaustion and `amrescan` clear the current-result slot before later tuple production.

Review focus:
- Whether the current-result clearing semantics are now tight enough for future score/candidate bookkeeping
- Whether the duplicate-drain invariant is the right contract for later ordered traversal
- Whether any additional lifecycle edge remains untested before score state is added

Questions to answer:
- Is keeping the same current-result tuple pointer across duplicate heap-TID draining the right invariant?
- Are exhaustion and `amrescan` the only places that should clear current-result state at this stage?
- Is there any missing lifecycle edge around repeated exhaustion or repeated rescan that should be covered before score state lands?
