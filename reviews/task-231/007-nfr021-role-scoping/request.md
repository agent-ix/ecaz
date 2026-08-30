---
task: 231
packet: 007-nfr021-role-scoping
agent: Codex
role: coder
model: GPT-5
date: 2026-08-30
seq: 1
---

# Task 231 NFR-021 role-scoping correction

Status: review-open. Code checkpoint: `795af9616a304f2bf276d57c2c151270198f9bd4`.
GitHub ticket: issue #97.

The accepted-SHA Task 231 decision suite completed all 27 configured steps, but
the final derived NFR-021 assertion exited nonzero. Every derived row reported
complete topology evidence, zero non-owned records, zero orphans, zero
unsharded derived bytes, zero coordinator-resident unsharded bytes, and a
constant head capacity. The sole violation was a normalized per-owner growth
ratio of `2.171204` against the `2.0` bound.

That ratio was an aggregation defect, not a measured distribution defect.
Both the current-heap control and fixed-stride candidate use the named
`production` search variant. `nfr_021_row_matches_variant` filtered physical
rows by variant but not by the already-emitted `nfr_021_role`, so each
registration combined the control's lower graph-side bytes with the
candidate's node-store bytes across scales. Every registration consequently
received the same false ratio.

The checkpoint adds a role check when a result row carries
`nfr_021_role`. It retains the historical variant-only fallback for rows from
steps without a registration, so suites that declare a registration once and
collect unlabeled matching steps keep their existing behavior. A focused
regression test proves that same-variant control rows match a control
registration, same-variant candidate rows do not, and unlabeled historical
rows retain the fallback.

No extension code, fixture command, suite config, measurement artifact, or
accepted measurement SHA changed. After review, the CLI will be rebuilt and
the unchanged all-succeeded suite manifest resumed; all 27 fixture steps must
be reused, with only `results.jsonl` and the derived conformance assertion
regenerated.

Please verify the role filter closes the cross-arm evidence contamination
without breaking the documented one-declaration cross-scale fallback.

