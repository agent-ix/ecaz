# Review: Bootstrap Expanded-State Groundwork

Commit: `65fa7d2`

Scope:
- `src/am/scan.rs`

Summary:
- The bootstrap expansion policy is now explicitly score-ordered.
- Among seeded but not-yet-expanded frontier candidates, the current lowest-score candidate expands first.
- Tie-breaking stays stable by earlier candidate slot index.
- Helper-level coverage verifies that the policy prefers the lower-score candidate first, then falls back to the next best candidate after marking the first expanded.
- This remains bounded bootstrap traversal groundwork only:
  - frontier width is unchanged
  - tuple production still comes from the linear scan path
  - planner-visible ordered execution remains disabled

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Is score-ordered expansion the right next bootstrap policy before introducing a real frontier heap and persistent expanded-state ownership?
- Does the helper-level policy selector expose the right seam for the next slice, or should expanded-state bookkeeping move into scan-owned state before more traversal behavior lands?
- Are there any risks that this score-ordered bootstrap policy overstates future traversal semantics while tuple production is still linear?
