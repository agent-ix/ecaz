---
agent: codex
role: coder
model: GPT-5
date: 2026-08-23
seq: 1
---

# Task 167 heldout gate semantics

Status: review-open. This packet addresses the gate-design correction required
by packet 059 feedback. No final scale-matrix or Task 167 closeout is claimed.

Code checkpoint `f58a69b41efbf5753b098b7476e7d7e7ba438c43` removes the
cross-scale `0.007` heldout acceptance constant and makes the heldout gate a
per-suite-step, per-scale regression detector:

- a step may supply the shipped-default heldout baseline deficit and the
  physical arm's sample standard deviation;
- the fixture computes `allowed_deficit = baseline_deficit + 2 * sample_sd`;
- both inputs are required together, finite, and non-negative;
- omitting both inputs records a non-blocking baseline observation with
  `quality_gate_mode=baseline_recording`,
  `quality_gate_applied=false`, and
  `disposition=disclosed_baseline_characteristic`;
- the FR-083-AC-4 inserted-neighborhood band remains a hard gate, and all
  measurement-integrity failures remain hard errors.

The accepted packet 059 insertion path is now labeled
`shipped_default_established_tie_priority` rather than a candidate. This is a
correctness-alignment disposition, not a quality claim. Packet 060 is retained
as an immutable preregistration record but will not run, per the reviewer.

## Validation

`cargo test -p ecaz-cli --no-default-features task167_` passed all 14 selected
tests with 498 filtered and no failures. The slice covers the baseline formula
and validation, suite-step expansion, structured result parsing, applied-gate
failure, non-applied baseline recording, and the existing Task 167 exact-recall
invariants. Durable output is in
[`artifacts/focused-tests.log`](artifacts/focused-tests.log).

## Review request

Please confirm that this implements packet 059 feedback §1 without restoring
an absolute heldout acceptance claim. After acceptance, the remaining work is
the final isolated 10k/50k/100k recall, latency, and storage matrix, recording
each scale's shipped-default heldout baseline as a disclosed characteristic.
