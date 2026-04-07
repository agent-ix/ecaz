# Request: Make Bootstrap-To-Linear Fallback Transition Explicit

Commit: `d42b59f`

Summary:
- Scan-owned state now records when the bootstrap phase is complete instead of retrying the bootstrap materialization path on every later `amgettuple` call.
- When bootstrap candidate adjudication drains the visible frontier without materializing a result, scan state now explicitly completes the bootstrap phase by clearing the visible frontier, scheduler, and expanded-source bookkeeping.
- The same cleanup now also happens on linear-scan exhaustion, and `amrescan` resets the phase back to bootstrap.

Files:
- `src/am/scan.rs`

Why this matters:
- After the recent consume -> adjudicate -> refill-on-success change, a `false` bootstrap materialization result now really means there are no visible bootstrap candidates left.
- Without an explicit phase transition, the executor would still keep re-entering the dead bootstrap path on later `amgettuple` calls before falling through to linear scan.
- This slice makes the current staged execution model more explicit and should be a better base for later ordered graph-search work.

Review focus:
- Whether `bootstrap_phase_complete` is the right current contract for the staged bootstrap-to-linear handoff
- Whether completing the bootstrap phase should clear all three pieces of bootstrap state: visible frontier, scheduler, and expanded-source bookkeeping
- Whether any runtime path can still legitimately need bootstrap re-entry after this phase bit flips, short of `amrescan`
