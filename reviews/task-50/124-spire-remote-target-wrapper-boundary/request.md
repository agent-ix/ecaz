---
task: 50
packet: 124-spire-remote-target-wrapper-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: 454fc95458438a3216dae33f05d49bdcd3f80e76
code_commit: 454fc95458438a3216dae33f05d49bdcd3f80e76
status: ready-for-review
---

# Review Request: SPIRE Remote Target Wrapper Boundary

## Summary

This slice removes the remaining direct unsafe wrapper blocks from
`src/am/ec_spire/coordinator/remote_candidates/fanout.rs`'s remote target
planning/readiness path.

The change makes these helpers safe:

- `remote_search_target_plan_rows`
- `remote_search_target_readiness_rows`

It also switches the corresponding SQL wrappers in `src/lib.rs` to
`with_live_index_relation_safe!` and removes now-unnecessary internal unsafe
calls from request plan/readiness helpers.

## Unsafe Burndown

- Previous packet count: `1635` unsafe blocks across `122` files.
- This packet count: `1632` unsafe blocks across `121` files.
- Net change: `-3` direct unsafe blocks and `-1` file with unsafe blocks.

The direct raw relation/page reads for this path remain centralized in the
module's lower-level helpers; these target/readiness wrappers only validate
inputs and compose safe SPIRE snapshot/planning helpers.

## Validation

Artifacts are under `reviews/task-50/124-spire-remote-target-wrapper-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1632 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1632`, `files 121`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
