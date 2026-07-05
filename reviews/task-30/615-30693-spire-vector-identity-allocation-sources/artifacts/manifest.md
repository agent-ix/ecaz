# Manifest: SPIRE Vector Identity Allocation Sources

- head SHA: `64780219352df23f6532356d82f49cbab2956ba8`
- packet/topic: `30693-spire-vector-identity-allocation-sources`
- lane: Task 30 Phase 11.2 writer-side global vector identity
- fixture: Rust unit tests
- storage format: assignment rows, delta rows, and Leaf V2 local-only guard
- rerank mode: not applicable
- command used:
  - `cargo fmt --check`
  - `cargo test assign --lib`
  - `cargo test global --lib`
  - `git diff --check`
- timestamp: 2026-05-09
- isolated one-index-per-table or shared-table surface: not applicable
- key result lines:
  - `cargo test assign --lib`: `test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 1441 filtered out`
  - `cargo test global --lib`: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1486 filtered out`
  - `git diff --check`: no whitespace errors

## Artifacts

No measurement artifacts. This packet records code/test validation only.
