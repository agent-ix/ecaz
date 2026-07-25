---
task: 36
agent: codex
role: coder
model: GPT-5
date: 2026-07-25
head: fbaa20a421eb3523caaccc92554df573f5771232
---

# Review request: close packet 004 minor follow-ups

Please review head `fbaa20a421eb3523caaccc92554df573f5771232`
against the two LOW follow-ups in packet 004.

## Corrections

- A new differential calls the production
  `candidate_batch::score_turboquant_qjl_batch_for` entry point at widths
  1/7/8/9/16/17/31/32/33. It checks the real block→octet→scalar cascade:
  SIMD blocks and octets use the documented 4-ULP/relative `1e-6` tolerance,
  while the sub-octet scalar remainder stays bit-exact.
- The older test-only QJL batch helper no longer contains an ISA-consistency
  assertion that could become a production panic path if the helper were
  promoted later.
- The counted Cargo wrapper now requires exactly one successful test-result
  summary. A future filtered stage that accidentally selects multiple test
  binaries fails closed instead of validating only the last summary.
- The hardening guide now explains the public-hook plus focused-guard split for
  `prod`, and accurately documents QJL octets as SIMD tolerance paths.
- The task record explicitly says the manual CI matrix has not been dispatched
  since counted `--lib` stages were added; CI-runner symbol resolution remains
  untested.

Intel and Graviton execution remain later host runs, as agreed. This packet
makes only an Apple arm64 NEON/dotprod claim.

## Evidence

See [artifacts/manifest.md](artifacts/manifest.md) for commands and provenance.

- [make-simd-diff.log](artifacts/make-simd-diff.log): all ten counted stages
  pass; the final three-test stage reports the production QJL cascade on NEON.
- [multiple-summary-negative-control.log](artifacts/multiple-summary-negative-control.log):
  two successful Cargo summaries are rejected with an observed count of two.
- [clippy-lib.log](artifacts/clippy-lib.log): the focused Clippy command reaches
  one unrelated pre-existing IVF `manual_checked_ops` finding; no touched Task
  36 file has a reported lint finding.

## Review focus

1. Does the new test exercise the production QJL block→octet→scalar composition
   with the correct exactness boundary?
2. Does the wrapper now reject both zero summaries and multiple summaries while
   retaining the existing expected-pass-count check?
3. Do the docs avoid claiming an undispatched CI run or hardware paths not
   executed on this host?

Task 36 remains review-requested pending an outside response.
