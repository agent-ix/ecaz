---
task: 92
packet: 002-graviton4-sve2-contract
agent: coder
date: 2026-06-09
---

# Task 92: Graviton 4 SVE2 Contract Retouch

## Summary

This checkpoint addresses the Task 92 Phase 1 reviewer F1 feedback before the
ADR-076 accepted-state flip.

Docs commit:

- `b4f847396f6cf60d3e6923e71d96f0be4c61794b`
  `Clarify ADR-076 Graviton 4 SVE2 target`

Changes:

- ADR-076 now distinguishes base `Sve` from `Sve2` in the proposed runtime ISA
  enum.
- The ARM measurement contract now names AWS Graviton 4 as the production
  target and says packets use the `sve2` dispatch branch when available.
- Width-specific evidence must report the measured runtime vector length
  verbatim; the concrete Graviton 4 example is `sve2-128`.
- The existing Task 92 Phase 1 request and skeleton audit were retouched to
  match the corrected ADR wording.

## Validation

See `artifacts/manifest.md` for artifact metadata.

No code tests were run for this docs-only ADR retouch.

## Review Focus

- Confirm the revised ADR wording no longer implies Graviton 4 is an SVE-256
  target.
- Confirm `Sve2` should be an explicit `Isa` variant for Task 92 implementation
  rather than hidden behind the base `Sve` branch.
