# Review Request: SPIRE Vector Identity Allocation Sources

## Summary

Phase 11.2 now has an explicit assignment-builder hook for local vs global
vector identity allocation.

Code checkpoint: `64780219352df23f6532356d82f49cbab2956ba8`
(`Add SPIRE vector identity allocation sources`)

## Scope

- Adds `SpireVecIdSourceIdentity` with:
  - default local `0x01` allocation;
  - caller-provided stable global payload allocation as `0x02`.
- Adds identity-aware primary, insert-delta, boundary, and
  boundary-insert-delta assignment builders.
- Preserves existing writer callers through local-default wrappers, so current
  build/insert behavior remains local-ID compatible.
- Ensures global identity allocation does not advance the local sequence.
- Ensures boundary replicas built with a global source identity share the same
  global `SpireVecId`.
- Tightens Leaf V2 local-only error wording and test coverage so the remaining
  global-ID storage blocker is explicit.
- Updates the Phase 11 task file to record the landed allocation hook and the
  remaining source-identity and Leaf V2 storage work.

## Validation

- `cargo fmt --check`
- `cargo test assign --lib`
  - 59 passed; 0 failed; 1441 filtered out
- `cargo test global --lib`
  - 14 passed; 0 failed; 1486 filtered out
- `git diff --check`

## Notes

This does not yet emit global IDs from live build/insert paths. The current AM
writer paths still lack a stable source-identity input, and Leaf V2 base
objects still reject global IDs because their vec-id column is fixed to the
local format. This checkpoint makes both boundaries explicit for the next
Phase 11.2 slice.
