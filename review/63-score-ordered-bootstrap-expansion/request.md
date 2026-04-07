# Review: Score-Ordered Bootstrap Expansion

Commit: `65fa7d2`

Scope:
- `src/am/scan.rs`

Summary:
- The explicit bootstrap expansion policy now changes behavior from insertion-order expansion to score-ordered expansion.
- Among seeded but not-yet-expanded candidates, the current lowest-score candidate expands first.
- Tie-breaking remains stable by earlier slot index.
- This is still bounded bootstrap traversal groundwork only:
  - frontier width is unchanged
  - tuple production still comes from the linear scan path
  - planner-visible ordered execution remains disabled
- Updated helper-level coverage now verifies:
  - the explicit score-ordered policy prefers the lower-score seeded candidate first
  - after the best candidate is marked expanded, the policy falls back to the next best candidate

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Is score-ordered seeded expansion the right next bootstrap policy, or does it create misleading traversal semantics before a real frontier heap exists?
- Is the tie-breaking behavior stable and coherent enough for this stage?
- Does this slice preserve a clean path toward a true scored frontier/visited traversal loop rather than baking in another temporary special case?
