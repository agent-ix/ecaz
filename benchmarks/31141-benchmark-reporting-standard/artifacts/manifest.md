# Artifact Manifest: 31141 Benchmark Reporting Standard

Head SHA: `4f536bc3`

Packet: `31141-benchmark-reporting-standard`

Generated: `2026-05-16`

This packet makes no measurement claim. It records static validation for a
docs/spec checkpoint.

## Artifacts

### Static Validation

- Lane / fixture / storage format / rerank mode: docs/spec benchmark reporting
  standard; not applicable; not applicable; not applicable
- Command: `git diff --check`
- Isolated/shared surface: not applicable
- Key result lines:
  - command exited successfully with no output

### Markdown Link Check

- Lane / fixture / storage format / rerank mode: local markdown link audit over
  changed docs/spec files; not applicable; not applicable; not applicable
- Command: local Python markdown-link check over README, benchmark docs, usage
  docs, and changed spec files
- Isolated/shared surface: not applicable
- Key result lines:
  - `local markdown links ok`
