---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 206 packet 001 — ceiling and axes: ACCEPT with notes

Reviewed commit `3fb1319af` against the packet claims. The phase-1 plumbing is
sound: `ECDISTANN_MAX_BEAM_WIDTH = 256` (`mod.rs:264`) is consumed by the GUC
registration and mirrored in the fixture/suite validators; `build_shards` is
threaded with matching `0..=4096` validation through the CLI arg
(`distann_multicluster.rs:109-110`), all three setup-SQL paths, and the suite
step (`suite.rs:684`, `:5318-5323`); and the debug-only NFR-019 cap is now a
runtime `Err` in release builds (`scan.rs:549-555`). BW/H are genuinely
first-class sweep axes via `benchmark_seed_variants`. Validation commands in
`artifacts/validation.md` are appropriate for a plumbing checkpoint, and the
packet correctly does not claim the measurement gate.

Notes (non-blocking for this packet):

- **P3 — NFR-019 check is a post-hoc tripwire, not an enforcement.** The
  `checked_mul` runs after the hop loop; the loop is structurally bounded
  (`for _round in 0..hop_rounds`, batch capped at `beam_width`,
  `scan.rs:376-403`), so this satisfies the audit intent, but the overflow
  branch is unreachable (`usize` at 256×256 = 65,536) and its "exceeds usize"
  message is misleading. Consider asserting the budget *before* the loop.
- **P3 — no end-to-end test exercises `beam_width = 256`.** The range-rejection
  test was updated (65→257), but nothing runs a scan at the new ceiling. A
  focused pgrx test at BW=256 would close the "asserted at the new values"
  claim properly.

The larger findings about what `build_shards` toggling actually measures are
Task 207 findings and are filed in
`reviews/task-207/001-construction-contract/feedback/2026-08-04-01-claude-reviewer.md`.
