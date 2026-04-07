# Request: Incremental Bootstrap Top-Up Scheduler

Commit: `712ac84`

Summary:
- Switches bootstrap frontier top-up in `src/am/scan.rs` onto one incremental `BeamSearch` scheduler instead of rebuilding source selection from scratch on each loop iteration.
- Keeps the existing frontier storage, expanded-source state, and visible scan behavior intact.

Files:
- `src/am/scan.rs`

Why this matters:
- This is the next real scan-side use of the shared `search` seam.
- It reduces duplicated score-order arbitration logic inside the executor while preserving the current bounded bootstrap behavior.
- It is the bridge toward letting scan execution reuse one search structure across repeated expansion steps.

Review focus:
- Whether the incremental scheduler preserves the current bootstrap expansion contract
- Whether the interaction between `expanded_source_tids` and the temporary beam scheduler is sound
- Whether this is the right next step before moving more frontier ownership out of `scan.rs`
