# Request: Bootstrap Frontier Top-Up After Consume

Commit: `f2321c5`

Summary:
- Changes post-consume bootstrap refill so it preserves expanded-source state and keeps topping the bounded frontier back up with the same score-ordered expansion policy used during initial seeding.
- This means a consumed candidate can still refill from its own adjacency first, but if that adds nothing unseen, another remaining unexpanded frontier candidate may now expand to restore bounded frontier width.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- The previous refill path was narrower than initial seeding and could stop early even when the remaining frontier still had expandable candidates.
- This keeps bounded bootstrap traversal behavior internally consistent without jumping to an unbounded traversal loop or planner-visible search.

Review focus:
- Whether expanded-source bookkeeping now has the right persistence boundary across rescan, initial seeding, and consume/refill
- Whether the consume/refill path can accidentally over-expand or re-expand prior sources
- Whether the updated pg and unit coverage captures both consumed-source refill and fallback refill from another remaining frontier candidate
