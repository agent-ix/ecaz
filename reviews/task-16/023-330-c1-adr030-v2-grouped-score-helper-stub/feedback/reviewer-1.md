## Feedback: ADR-030 v2 Grouped Score Helper Stub

Read `score_grouped_candidate_input(...)` (renamed to `score_grouped_candidate_context`
in packet 333).

### What's right

- Helper extraction into its own function is the correct shape. The future scorer
  replaces one function body rather than surgically editing dispatch and helper
  together. Packets 331/332/333 then progressively reshape the helper's inputs
  without touching dispatch call sites — that payoff is visible.
- The testing note is the right decision: a direct unit test on a helper that panics
  through `pgrx::error!` fights the runtime rather than testing intent. Shape tests at
  the dispatch level are the right alternative.

### Concern

Leaving a stub helper that only raises an error creates a subtle readability cost: a
reader has to trace through two levels of indirection (dispatch → helper → panic) to
see that grouped scoring is unsupported. Document this inside the helper — one short
line like "stub until the grouped scorer lands in packet N" — so the indirection is
intentional, not accidental.

Small point; do not block progress on it.

### Observation

Packets 329-330 are a good example of how to introduce a future-proof cut point
without committing to the implementation. The stub exists, is reachable, and errors
loudly. The next three packets then shape its inputs. That's the correct order.
