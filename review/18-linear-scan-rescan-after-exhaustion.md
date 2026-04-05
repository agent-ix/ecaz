# Review Request: Linear Scan Rescan After Exhaustion

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added explicit regression coverage for calling `amrescan` after the current linear scan has fully exhausted all tuples.
- The new coverage verifies that a full exhausted pass returns every expected heap TID, then a subsequent `amrescan` restarts tuple production from the beginning.
- Added a small test-only type alias cleanup while wiring the new debug helper to keep `clippy` green.

Review focus:
- Whether restarting from the beginning after a full exhausted pass is the right current `amrescan` contract for the bootstrap linear scan
- Whether the new debug helper matches the current scan-state ownership and reset boundaries cleanly
- Whether this coverage is enough for the exhausted-then-rescanned path, or still misses an important lifecycle edge

Questions to answer:
- Is there any missing regression around rescanning after exhaustion when the final returned tuple came from a duplicate-coalesced element rather than a singleton element?
- Is there any stale-state risk left after exhaustion plus rescan, beyond what this coverage exercises?
- Should this exhausted-rescan path be called out explicitly in the linear-scan ADR, or is review coverage enough for now?
