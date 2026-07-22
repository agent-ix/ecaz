# Task 192 pre-registration manifest

- Candidate: cached resolved row-tier schema in the existing four-entry
  per-backend, per-generation retained-epoch LRU.
- Code head: `task-192-194-owner-transport` (record exact review commit).
- Scope: owner endpoint validation only; no protocol, payload SQL, or
  traversal behavior change.
- Validation command: `cargo check --offline -p ecaz --lib --no-default-features --features pg18`.
- Required next evidence: paired 100k A/B recall, latency, storage, and epoch
  fencing drill via `ecaz bench suite`.
