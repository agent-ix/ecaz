# Review Request: SPIRE Nprobe Parsed Options

Code checkpoint: `b736da35` (`Store SPIRE per-level nprobe parsed options`)

## Summary

This slice addresses reviewer finding F-NPROBE-1 from
`30656-spire-per-level-nprobe`: `nprobe_per_level` was parsed once for
reloption validation and then parsed again when scan policy consumed
`EcSpireOptions`. The options object now stores the parsed `Vec<u32>` directly,
so scan planning reuses the validated values instead of splitting/parsing the
string again.

## Scope

- Changes `EcSpireOptions::nprobe_per_level` from `Option<String>` to
  `Option<Vec<u32>>`.
- Keeps reloption validation fail-closed at `relation_options(...)` read time.
- Simplifies `nprobe_per_level_values()` to return the already-parsed values.
- Updates direct unit-test fixtures that construct `EcSpireOptions` by hand.
- Processes reviewer F-NPROBE-2 by clarifying in the Phase 3 closeout task that
  per-level fanout is build-time children-per-parent, while `nprobe_per_level`
  is scan-time children visited per parent.

## Validation

Packet-local logs live under `artifacts/` and are indexed in
`artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`
  - `Finished dev profile ... target(s) in 0.17s`
- `cargo fmt --check`
  - exited `0`; existing stable-rustfmt warnings for unstable options remain
- `cargo test --no-default-features --features pg18 nprobe_per_level`
  - `1 passed; 0 failed`
- `cargo test --no-default-features --features pg18 per_level_nprobe`
  - `3 passed; 0 failed`
- `cargo test --no-default-features --features pg18 collect_scan_routing_diagnostics -- --test-threads=1`
  - `2 passed; 0 failed`
- `git diff --check`
  - exited `0`

## Review Questions

- Is storing `Option<Vec<u32>>` in `EcSpireOptions` the right narrow fix, or
  should this be moved to the fixed-array `SpireRecursiveNprobePolicy` shape at
  relation-option read time?
- Is keeping `nprobe_per_level_values()` as a cloning accessor acceptable for
  this stage, given that parsing is removed but a small Vec clone remains?
- Does the Phase 3 fanout-vs-nprobe note resolve the closeout ambiguity without
  prematurely flipping the fanout checkbox?
