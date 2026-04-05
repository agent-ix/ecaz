# Review Request: Two-Slot Frontier Head Ordering

Scope:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added one explicit head-selection rule for the existing two-slot candidate frontier.
- The rule is intentionally small: valid candidates beat empty slots, and when both slots are valid the lower score wins.
- The frontier still is not a general queue or heap; this slice only makes the current two-slot structure pick a best slot deterministically.
- Added regression coverage that the reported frontier head matches that ordering rule for the current seeded entry/successor frontier.

Review focus:
- Whether this head-selection rule is the right next piece of explicit ordering semantics before a larger frontier exists
- Whether score-based selection across the two slots is clear and low-risk at the current stage
- Whether the test captures the rule precisely enough without pretending we have a full traversal queue

Questions to answer:
- Is `valid first, then lower score` the right ordering rule for the current two-slot frontier?
- Should the frontier head live as explicit state now, or would deriving it lazily be cleaner until a real queue exists?
- Are there any missing lifecycle edges around head recomputation on rescan or exhaustion that should be covered before the next slice?
