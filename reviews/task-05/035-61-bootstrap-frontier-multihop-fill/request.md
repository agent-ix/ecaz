# Review: Bootstrap Frontier Multi-Hop Fill

Commit: `f9b5c9b`

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

Summary:
- Bootstrap frontier seeding no longer stops after expanding only the entry point.
- During `amrescan`, the bounded bootstrap frontier now keeps expanding from already seeded candidates in frontier order until:
  - the current bootstrap frontier width is full, or
  - there are no more seeded candidates left to expand
- This is still intentionally bounded traversal groundwork:
  - current frontier width stays capped at the existing bootstrap limit
  - tuple production still comes from the linear scan path
  - planner-visible ordered execution remains disabled
- Added a pure unit test for the helper-level contract:
  - a frontier seeded with an entry candidate can pick up a child and then a grandchild across successive seeded sources
- Updated pg coverage to validate:
  - bootstrap frontier shape is bounded rather than tied strictly to immediate entry-neighbor count
  - provenance remains coherent when later seeded candidates come from other seeded candidates instead of directly from the entry point

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Is the new bounded multi-hop bootstrap fill the right next traversal seam, or does it introduce hidden assumptions before a real frontier/visited/traversal loop exists?
- Does expanding seeded candidates in frontier insertion order make sense for this bootstrap stage, or should the next slice move that logic behind an explicit traversal policy?
- Are the updated pg assertions and helper unit test strong enough to cover the new behavior without overfitting to one graph shape?
