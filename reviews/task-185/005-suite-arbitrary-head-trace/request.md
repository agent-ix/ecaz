---
task: 185
packet: 005-suite-arbitrary-head-trace
role: coder
status: open
date: 2026-08-07
seq: 01
head: 9627d36c2
---

# Task 185 suite wiring checkpoint

This packet wires the arbitrary persisted-head attribution endpoint from
packet 004 into the canonical `ecaz bench suite` runner. It still does not
claim a selector, an A/B result, or a production change.

## Change

Commit `9627d36c2` adds two explicit suite options:

- `gateway_head_candidate_trace` enables the diagnostic;
- `gateway_head_candidate_positions` supplies bounded, 1-based persisted-head
  positions to trace per disjoint training query.

The CLI validates the physical benchmark and training-slice prerequisites,
checks positions are within `1..=4096`, preflights the new extension endpoint,
and writes one packet-directed JSON trace per physical variant. The suite
artifact manifest includes those JSON outputs, so a future run is resumable
and review-visible.

The diagnostic uses the same physical settings as the control and invokes the
feature-only endpoint from packet 004. It does not modify the production seed
selector or head format.

## Validation

- PG18 extension feature check with
  `distann-head-attribution-benchmark`: pass, exit 0.
- PG18 `ecaz-cli` check: pass, exit 0; one pre-existing dead-code warning.
- `git diff --check`: pass.
- No fixture or benchmark run was performed in this checkpoint.

The next step is to build/install the feature extension and run a bounded
100k training diagnostic through `ecaz bench suite` (for example, a sparse
position set before attempting a larger candidate screen). Only after that
diagnostic proves useful should a selector implementation or fixed-cap A/B
arm be considered.

## Review focus

Please review the suite validation, position bounds, SQL parameter construction,
artifact registration, and the fact that this remains an explicit diagnostic
rather than a default or persisted policy.
