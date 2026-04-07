# Review: Bootstrap Expansion Policy Seam

Commit: `be40792`

Scope:
- `src/am/scan.rs`

Summary:
- Bootstrap frontier fill now routes through an explicit `BootstrapExpandPolicy` selector.
- The current behavior is preserved as `BootstrapExpandPolicy::InsertionOrder`; this slice is structural, not a traversal-policy change.
- `fill_bootstrap_frontier` no longer relies on ad hoc local index advancement. It now asks `next_bootstrap_expand_index(...)` for the next seeded source to expand.
- Added unit coverage for:
  - bounded multi-hop fill still working through the explicit policy seam
  - the current insertion-order policy choosing the first unexpanded seeded candidate, then advancing to the next one

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Is the policy seam at the right level for upcoming traversal work, or should the next slice move policy selection closer to frontier-head scoring/expansion state?
- Does preserving `InsertionOrder` as the current explicit policy keep the bootstrap behavior clear enough while still giving the next slice a clean place to switch policy?
- Are the new helper-level tests sufficient to protect the seam before a real scored expansion policy lands?
