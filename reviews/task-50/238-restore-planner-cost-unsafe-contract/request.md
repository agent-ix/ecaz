---
task: 50
packet: 238
topic: restore-planner-cost-unsafe-contract
role: coder
status: ready-for-review
created: 2026-05-21T05:41:04-07:00
head_sha: f4c9aa753112cf1270db7322babe9ad50227795b
addresses_feedback:
  - reviews/task-50/236-planner-cost-global-accessors/feedback/2026-05-21-01-reviewer.md
---

# Review Request: Restore Planner Cost Unsafe Contract

## Summary

This packet addresses the blocking feedback on packet 236.

Changes:

- Restored `current_planner_cost_constants` to `pub(crate) unsafe fn`.
- Restored `current_cpu_tuple_cost` to `pub(crate) unsafe fn`.
- Restored explicit `unsafe { ... }` call sites and SAFETY comments in IVF, SPIRE, SPIRE custom scan, DiskANN, HNSW, and common HNSW cost paths.
- Removed the packet-236 doc text that asserted these reads impose no memory-safety precondition on callers.

## Feedback Assessment

The reviewer was correct for this repository's current audit convention and process:

- Packet 236 reversed a previously closed round-1 audit decision from packet 141 without asking for user adjudication.
- The repository convention treats PostgreSQL backend-context-only reads as unsafe contracts that callers must explicitly acknowledge.
- The fix here follows the reviewer's default Path A: keep the round-1 decision.

## Unsafe Count

- Previous repo count before this fix: `2478`
- Current repo count after this fix: `2490`
- Delta: `+12`

This is an intentional count increase to restore the accepted safety contract and unblock packet 236 feedback.

The packet-local count and contract log is:

- `artifacts/unsafe-contract-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: scoped `rustfmt --check` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-cost-pg18-no-run.log`: `cargo test --lib cost --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
