# Request: Cover Bootstrap-Phase Transition At The PG Surface

Commit: `7e5d15e`

Summary:
- Scan debug helpers now expose the current bootstrap-phase transition as one explicit lifecycle check instead of requiring outside tests to infer it through lower-level frontier helpers.
- The new pg regression proves that a non-empty scan eventually marks `bootstrap_phase_complete`, clears the visible frontier head/slots at that point, and resets the phase bit on `amrescan`.
- This keeps the new staged bootstrap-to-linear handoff visible at the same pg/debug surface the rest of the executor lifecycle tests already use.

Files:
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- `d42b59f` made bootstrap completion an explicit runtime contract, but only unit tests were pinning that behavior.
- Without pg/debug coverage, later scan-path refactors could regress the phase transition while still leaving helper-level state tests green.
- This slice makes the bootstrap-to-linear handoff observable through the same external lifecycle surface that already covers rescan, frontier state, and tuple production.

Review focus:
- Whether the new debug helper exercises the same completion path that runtime `amgettuple` uses
- Whether the asserted post-bootstrap state is the right public contract for the current staged executor
- Whether `amrescan` now resets all of the phase-transition state that outside tests should care about
