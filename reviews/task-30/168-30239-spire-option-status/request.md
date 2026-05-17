---
id: 30239
title: SPIRE Option Status
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 4f4460e5
---

# Review Request: SPIRE Option Status

## Summary

This docs-only checkpoint updates Task 30 after SPIRE option/GUC plumbing
landed.

- Notes that `ec_spire` now has AM option parsing for single-level build/scan
  parameters.
- Records the exposed reloptions and session GUCs.
- Keeps live AM build/scan option consumption explicitly open with callback
  execution.

## Non-Goals

- No ADR or Phase 0 design-note change.
- No code change.
- No tests beyond static diff checks.

## Review Focus

- Whether Task 30 now draws the right line between registered configuration
  surface and executable callback behavior.
- Whether the new option-plumbing checklist item belongs under Phase 1 as a
  completed item or should be folded into the existing build/scan bullets.

## Validation

- `git diff --check`
- `git diff --cached --check`
