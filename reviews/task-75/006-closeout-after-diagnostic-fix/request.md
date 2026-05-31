---
task: 75
topic: closeout-after-diagnostic-fix
agent: codex
role: coder
model: GPT-5
date: 2026-05-31
---

# Task 75 Closeout After Diagnostic Fix

## Request

Please review this reissued Task 75 closeout. It replaces `reviews/task-75/003-closeout/`, which was held open because packet 001 used a flawed diagnostic path.

## What Changed

- Fixed `scan::collect_single_level_scan_placement_diagnostics` so SQL-visible local pipeline diagnostics use top-graph routing when the production scan would use top-graph routing.
- Re-ran the Task 75 Intel-local routing envelope suite after installing the fixed PG18 extension.
- Reissued the Phase 2 decision using corrected candidate counts.
- Added Task 77 as the optimization follow-up for non-routing candidate materialization/scoring work.

## Closeout Position

Task 75 should close after reviewer acceptance of packets 004-006:

- Phase 1 rerun is complete and uses `ecaz bench suite`.
- The corrected funnel is no longer query-invariant at tg32+ and reconciles with suite counters.
- Phase 2 routing slices are shelved because no semantics-preserving routing predicate is supported by the evidence.
- Remaining plausible optimization work is now scoped to Task 77, outside Task 75's routing-envelope decision.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`
  - `reviews/task-75/004-diagnostic-fix-rerun/artifacts/cargo-test-spire-pipeline.log`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `artifacts/cargo-clippy-pg18.log`
- `git diff --check`
  - `artifacts/git-diff-check.log`
- no new `unsafe` in the scan diff
  - `artifacts/no-new-unsafe-scan.log`
- `ecaz bench suite audit`, dry-run, run, and report
  - `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/`

## Task Status

`plan/tasks/75-spire-latency-routing-envelope.md` is set to pending reviewer acceptance rather than complete. On acceptance, it can flip back to complete with this packet as the closeout reference.
