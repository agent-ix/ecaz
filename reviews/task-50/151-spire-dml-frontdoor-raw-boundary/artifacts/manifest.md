---
task: 50
packet: reviews/task-50/151-spire-dml-frontdoor-raw-boundary
head_sha: 34405b854f8b6b72208ac64425e0b80fab20c435
created_at: 2026-05-20T20:38:48-07:00
---

# Manifest

## Scope

SPIRE round-3 soundness audit follow-up for DML frontdoor helpers that take
raw planner and executor parameter pointers.

## Artifacts

- `cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: pass
  - Key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
  - Note: pre-existing unused-import warning in `src/am/mod.rs`.
- `git-diff-check.log`
  - Command: `git diff --check`
  - Result: pass; empty output.
- `unsafe-block-count.log`
  - Command: `make unsafe-block-count`
  - Result: pass
  - Key result: summed count is 1641.
