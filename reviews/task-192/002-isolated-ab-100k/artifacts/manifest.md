# Task 192 isolated-candidate manifest

- Candidate advanced: none.
- Reason: no completed paired benchmark and unresolved live-schema invalidation
  semantics.
- Validation: `cargo check --offline -p ecaz --lib --no-default-features
  --features pg18` passed after reverting the unsafe cache extension.
